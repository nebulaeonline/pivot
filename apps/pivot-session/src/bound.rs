//! Production bound views and explicitly authorized fixture refresh.
use super::*;
use pivot_workbook::producers::{self, AttemptStatus, Fixture, KeyPolicy, ProducerId, Source};
fn producer_id(text: &str) -> Result<ProducerId, String> {
    Ok(ProducerId(
        Uuid::parse_str(text).map_err(|_| "Producer ID required")?,
    ))
}
fn fixture(text: &str) -> Result<Fixture, String> {
    match text {
        "ledger" => Ok(Fixture::Ledger),
        "typed" => Ok(Fixture::Typed),
        _ => Err("Choose an installed fixture: ledger or typed".into()),
    }
}
fn status(status: AttemptStatus) -> &'static str {
    match status {
        AttemptStatus::Never => "Not refreshed",
        AttemptStatus::InProgress => "Refreshing",
        AttemptStatus::Succeeded => "Succeeded",
        AttemptStatus::Failed => "Failed; retained snapshot",
        AttemptStatus::Cancelled => "Cancelled; retained snapshot",
        AttemptStatus::Disconnected => "Disconnected",
        AttemptStatus::Interrupted => "Interrupted on reopen",
    }
}
pub(super) fn info(p: &producers::Producer) -> wire::ProducerInfo {
    let (rows, columns) = p.extent();
    let mut out = wire::ProducerInfo {
        id: p.id.0.to_string(),
        configuration: p.configuration,
        rows,
        columns,
        retired: p.retired,
        ..Default::default()
    };
    match &p.source {
        Source::Worksheet {
            worksheet,
            rectangle,
            ..
        } => {
            out.kind = "Live worksheet".into();
            out.source_worksheet = worksheet.uuid().to_string();
            if let Some(r) = rectangle {
                out.first = Some(location(r.first));
                out.last = Some(location(r.last));
            }
            out.refresh_status = if rectangle.is_some() {
                "Live"
            } else {
                "Source unavailable"
            }
            .into();
        }
        Source::Fixture {
            fixture,
            schema,
            policy,
            selected,
            attempt,
        } => {
            out.kind = "Explicit refresh".into();
            out.fixture = fixture.name().into();
            out.schema = schema.id.0.to_string();
            out.generation_qualified = *policy == KeyPolicy::GenerationQualified;
            out.selected_generation = selected
                .as_ref()
                .map(|s| s.generation.0.to_string())
                .unwrap_or_default();
            out.refresh_status = status(attempt.status).into();
            out.last_outcome = status(attempt.last_outcome).into();
            out.fields = schema
                .fields
                .iter()
                .map(|f| wire::ProducerField {
                    id: f.id.0.to_string(),
                    label: f.label.clone(),
                    kind: format!("{:?}", f.kind),
                    nullable: f.nullable,
                })
                .collect();
        }
    }
    if p.retired {
        out.refresh_status = "Source unavailable".into();
    }
    out
}
pub(super) fn window_info(book: &Workbook, s: &pivot_workbook::SheetInfo) -> wire::WorksheetInfo {
    let mut out = wire::WorksheetInfo {
        id: s.id.uuid().to_string(),
        name: s.name.clone(),
        rows: s.row_count(),
        columns: s.column_count(),
        ..Default::default()
    };
    if let Some(binding) = book.window_binding(s.id) {
        out.bound = true;
        out.producer = binding.producer.0.to_string();
        if let Some(p) = book.producers().iter().find(|p| p.id == binding.producer) {
            let p = info(p);
            out.binding_mode = p.kind;
            out.selected_generation = p.selected_generation;
            out.refresh_status = p.refresh_status;
        }
    }
    out
}
fn emit_frame(
    r: &mut wire::Response,
    frame: producers::ProjectedFrame,
    capture: &CalculationCapture,
) -> Result<(), String> {
    r.bound_projection = true;
    r.worksheet = frame.window.uuid().to_string();
    r.window_scope = true;
    r.producer_id = frame.producer.0.to_string();
    r.producer_configuration = frame.configuration;
    r.producer_generation = frame
        .generation
        .map(|g| g.0.to_string())
        .unwrap_or_default();
    r.bound_rows = frame.rows;
    r.bound_columns = frame.columns;
    r.producer_status = if let Some(p) = capture.producer(frame.producer) {
        info(p).refresh_status
    } else {
        "Source unavailable".into()
    };
    for source in frame.cells {
        let mut cell = wire::Cell {
            at: Some(location(source.position)),
            worksheet: frame.window.uuid().to_string(),
            projected: Some(wire::ProjectedLocation {
                window: frame.window.uuid().to_string(),
                producer: frame.producer.0.to_string(),
                configuration: frame.configuration,
                generation: r.producer_generation.clone(),
                item_key: source.item_key,
                field_id: source.field_id,
                source_cell: source
                    .source
                    .map(|c| c.cell.uuid().to_string())
                    .unwrap_or_default(),
                source_hidden: source.source_hidden,
            }),
            ..Default::default()
        };
        match source.value {
            Ok(value) => {
                set_value(&mut cell, value.clone());
                cell.content = Some(match value {
                    pivot_types::PivotValue::Blank => wire::cell::Content::Blank(true),
                    pivot_types::PivotValue::String(s) => wire::cell::Content::Text(s),
                    pivot_types::PivotValue::Boolean(b) => wire::cell::Content::Boolean(b),
                    pivot_types::PivotValue::Decimal(d) => {
                        let d = source
                            .decimal
                            .unwrap_or_else(|| DecimalInput::from_decimal(d));
                        let (lo, mid, hi, negative, scale) = d.parts();
                        let decimal = Decimal128 {
                            lo,
                            mid,
                            hi,
                            negative,
                            scale,
                        };
                        cell.value = Some(decimal);
                        cell.result = Some(wire::TypedValue {
                            kind: Some(wire::typed_value::Kind::Decimal(decimal)),
                        });
                        cell.formatted_number =
                            pivot_workbook::formatting::render_decimal(d.value(), source.style)
                                .map_err(|e| e.to_string())?;
                        wire::cell::Content::Number(decimal)
                    }
                    _ => return Err("Unsupported producer value".into()),
                });
            }
            Err(e) => {
                structured::set_cell_fault(&mut cell, &e, capture);
                cell.error_code = error_code(&e);
                cell.error = readable_error(format!("{e:?}"));
            }
        }
        r.styles.push(wire::StyledPosition {
            at: Some(location(source.position)),
            style: source.style.encode().to_vec(),
        });
        r.cells.push(cell);
    }
    Ok(())
}
impl Session {
    pub(super) fn handle_bound(
        &mut self,
        q: &wire::Request,
        r: &mut wire::Response,
        op: wire::request::Operation,
    ) -> Result<bool, String> {
        use wire::request::Operation as Op;
        let producer_operation = matches!(
            op,
            Op::CreateWorksheetProducer
                | Op::CreateFixtureProducer
                | Op::CreateBoundWindow
                | Op::BeginProducerRefresh
                | Op::CompleteProducerRefresh
                | Op::FailProducerRefresh
                | Op::CancelProducerRefresh
                | Op::DisconnectProducer
                | Op::ReconnectProducer
                | Op::PrepareProducerDelete
                | Op::DeleteProducer
                | Op::DetachBoundWindow
                | Op::RefreshProducer
        );
        let scope = if matches!(
            op,
            Op::Read | Op::ReadFiltered | Op::ReadStyles | Op::Calculate
        ) {
            request_scope(q).ok()
        } else {
            None
        };
        let bound = scope.is_some_and(|scope| {
            self.book
                .as_ref()
                .is_some_and(|b| b.window_binding(scope).is_some())
        });
        if !producer_operation && !bound {
            return Ok(false);
        }
        if self.proof.is_some() {
            return Err("Use an ordinary workbook for producers.".into());
        }
        if op == Op::Calculate && bound {
            if q.outputs.is_empty() || q.outputs.len() > MAX_CELLS {
                return Err("1..256 outputs required".into());
            }
            let capture = self.captures.get(&q.capture).ok_or("Unknown capture")?;
            if Some(capture.scope) != scope {
                return Err("Capture scope mismatch".into());
            }
            let GridScopeId::Window(window) = capture.scope else {
                return Err("Window required".into());
            };
            r.revision = capture.revision.clone();
            r.capture_state = capture.computational_state.clone();
            r.capture_identity = q.capture.clone();
            r.generation = q.capture.clone();
            for p in &q.outputs {
                let p = pos(Some(p))?;
                let frame = capture
                    .value
                    .project_window(window, Rectangle { first: p, last: p })
                    .map_err(|e| e.to_string())?;
                emit_frame(r, frame, &capture.value)?;
            }
            return Ok(true);
        }
        if q.revision != self.revision {
            r.conflict = true;
            return Err("expected-state conflict".into());
        }
        let book = self.book.as_mut().ok_or("Create workbook first")?;
        let before = book.expected();
        match op {
            Op::CreateWorksheetProducer => {
                let sheet = pivot_workbook::WorksheetId::from_uuid(
                    Uuid::parse_str(&q.source_worksheet)
                        .map_err(|_| "Source worksheet ID required")?,
                );
                r.producer_id = book
                    .create_worksheet_producer(
                        before,
                        sheet,
                        Rectangle {
                            first: pos(q.first.as_ref())?,
                            last: pos(q.last.as_ref())?,
                        },
                    )
                    .map_err(|e| e.to_string())?
                    .0
                    .to_string();
            }
            Op::CreateFixtureProducer => {
                r.producer_id = book
                    .create_fixture_producer(
                        before,
                        fixture(&q.fixture)?,
                        if q.generation_qualified {
                            KeyPolicy::GenerationQualified
                        } else {
                            KeyPolicy::Durable
                        },
                    )
                    .map_err(|e| e.to_string())?
                    .0
                    .to_string();
            }
            Op::CreateBoundWindow => {
                let id = book
                    .create_bound_window(before, &q.sheet_name, producer_id(&q.producer_id)?)
                    .map_err(|e| e.to_string())?;
                r.worksheet = id.uuid().to_string();
                r.window_scope = true;
            }
            Op::BeginProducerRefresh => {
                let id = producer_id(&q.producer_id)?;
                let ticket = book.begin_refresh(before, id).map_err(|e| e.to_string())?;
                let token = Uuid::new_v4().to_string();
                self.producer_refresh.insert(token.clone(), (id, ticket));
                r.producer_ticket = token;
            }
            Op::CompleteProducerRefresh | Op::FailProducerRefresh | Op::CancelProducerRefresh => {
                let id = producer_id(&q.producer_id)?;
                if self
                    .producer_refresh
                    .get(&q.producer_ticket)
                    .is_none_or(|(producer, _)| *producer != id)
                {
                    return Err("Unknown refresh ticket".into());
                }
                let (_, ticket) = self
                    .producer_refresh
                    .remove(&q.producer_ticket)
                    .ok_or("Unknown refresh ticket")?;
                if op == Op::CompleteProducerRefresh {
                    let result = book.refresh_fixture(&ticket, q.fixture_version);
                    if result.is_err() {
                        let _ = book.finish_refresh(&ticket, None, false);
                    }
                    if book.expected() != before {
                        self.revision = Uuid::new_v4().to_string();
                    }
                    r.revision = self.revision.clone();
                    result.map_err(|e| e.to_string())?;
                } else {
                    book.finish_refresh(&ticket, None, op == Op::CancelProducerRefresh)
                        .map_err(|e| e.to_string())?;
                }
            }
            Op::RefreshProducer => {
                let ticket = book
                    .begin_refresh(before, producer_id(&q.producer_id)?)
                    .map_err(|e| e.to_string())?;
                let result = book.refresh_fixture(&ticket, q.fixture_version);
                if result.is_err() {
                    let _ = book.finish_refresh(&ticket, None, false);
                }
                if book.expected() != before {
                    self.revision = Uuid::new_v4().to_string();
                }
                r.revision = self.revision.clone();
                result.map_err(|e| e.to_string())?;
            }
            Op::DisconnectProducer => {
                book.disconnect_producer(before, producer_id(&q.producer_id)?)
                    .map_err(|e| e.to_string())?;
            }
            Op::ReconnectProducer => {
                book.reconnect_producer(before, producer_id(&q.producer_id)?, fixture(&q.fixture)?)
                    .map_err(|e| e.to_string())?;
            }
            Op::PrepareProducerDelete => {
                let id = producer_id(&q.producer_id)?;
                r.producer_inbound_references =
                    book.producer_inbound_references(id, &Default::default())
                        .map_err(|e| e.to_string())? as u64;
                let ticket = Uuid::new_v4().to_string();
                self.producer_delete = Some((ticket.clone(), before, id));
                r.producer_delete_ticket = ticket;
            }
            Op::DeleteProducer => {
                let id = producer_id(&q.producer_id)?;
                if self.producer_delete.as_ref()
                    != Some(&(q.producer_delete_ticket.clone(), before, id))
                {
                    return Err("Producer deletion review is stale; review again".into());
                }
                book.retire_producer(before, id, &Default::default())
                    .map_err(|e| e.to_string())?;
                self.producer_delete = None;
            }
            Op::DetachBoundWindow => {
                let GridScopeId::Window(window) = request_scope(q)? else {
                    return Err("Bound Window required".into());
                };
                let id = book
                    .detach_bound_window(before, window, &q.sheet_name, &Default::default())
                    .map_err(|e| e.to_string())?;
                r.worksheet = id.uuid().to_string();
                r.window_scope = true;
            }
            Op::Read | Op::ReadStyles | Op::ReadFiltered => {
                if op == Op::ReadStyles && q.style_summary {
                    return Err("Bound Windows expose source styles in projected frames; they have no independent formatting summary or defaults".into());
                }
                let GridScopeId::Window(window) = scope.ok_or("Window required")? else {
                    return Err("Window required".into());
                };
                let capture = book
                    .capture_sheet(window.into(), before)
                    .map_err(|e| e.to_string())?;
                let binding = book.window_binding(window.into()).ok_or("Not bound")?;
                let producer = book
                    .producers()
                    .iter()
                    .find(|p| p.id == binding.producer)
                    .ok_or("Source unavailable")?;
                let (rows, columns) = producer.extent();
                let (first, last) = if op == Op::ReadFiltered {
                    if q.filter_row_count == 0 || q.filter_row_count > 20 {
                        return Err("Viewport requires 1..20 rows".into());
                    }
                    let first = q.filter_first_visible.max(1).min(rows.max(1));
                    (
                        Position::new(first, pos(q.first.as_ref())?.column),
                        Position::new(
                            (first + q.filter_row_count - 1).min(rows.max(1)),
                            pos(q.last.as_ref())?.column.min(columns.max(1)),
                        ),
                    )
                } else {
                    (pos(q.first.as_ref())?, pos(q.last.as_ref())?)
                };
                let count = u64::from(last.row.saturating_sub(first.row) + 1)
                    * u64::from(last.column.saturating_sub(first.column) + 1);
                if count > MAX_CELLS as u64 {
                    return Err("Viewport exceeds 256 cells".into());
                }
                let frame = capture
                    .project_window(window, Rectangle { first, last })
                    .map_err(|e| e.to_string())?;
                r.capture_state = self.computational_state.clone();
                r.generation = Uuid::new_v4().to_string();
                r.capture_identity = r.generation.clone();
                if op == Op::ReadFiltered {
                    let extent = rows.max(1);
                    let mut mask = vec![0u8; 8192];
                    for row in 1..=extent {
                        mask[(row as usize - 1) / 8] |= 1 << ((row - 1) % 8);
                    }
                    r.filter_visible_mask = mask;
                    r.filter_total_visible_rows = extent;
                    r.filter_first_visible = first.row;
                    r.filter_rows = (first.row..=last.row).collect();
                }
                emit_frame(r, frame, &capture)?;
            }
            _ => return Ok(false),
        }
        if book.expected() != before {
            self.revision = Uuid::new_v4().to_string();
        }
        r.revision = self.revision.clone();
        Ok(true)
    }
}
