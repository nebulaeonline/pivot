//! Stateless presentation commands qualified by explicit scope/canonical capture.
use super::*;
use pivot_workbook::geometry::{CellRect, Command, SizeRun};
pub(super) struct Lease {
    map: pivot_workbook::geometry::Map,
    viewport: Option<CellRect>,
    payload: Option<pivot_core::model::integration::resources::Charge>,
    emitted_bytes: usize,
    filter: wire::Response,
    layout_stamp: Option<[u8; 32]>,
    layout_needs_review: bool,
}
fn emit_cell(
    capture: &CalculationCapture,
    scope: GridScopeId,
    position: Position,
    r: &mut wire::Response,
) -> Result<(), String> {
    if let GridScopeId::Window(window) = scope
        && capture
            .positional_geometry(scope)
            .map_err(|e| e.to_string())?
    {
        let frame = capture
            .project_window(
                window,
                Rectangle {
                    first: position,
                    last: position,
                },
            )
            .map_err(|e| e.to_string())?;
        return bound::emit_frame(r, frame, capture);
    }
    let style = capture
        .style_at(scope, position)
        .map_err(|e| e.to_string())?;
    let mut view = wire::Cell {
        at: Some(location(position)),
        worksheet: scope.uuid().to_string(),
        ..Default::default()
    };
    if let Some(cell) = capture.captured_cell(position).map_err(|e| e.to_string())? {
        view.cell_id = cell.reference.cell.uuid().to_string();
        view.node_id = cell.node.uuid().to_string();
        let effective = capture
            .captured_formula(cell.reference)
            .map_err(|e| e.to_string())?;
        view.content = Some(match cell.content {
            Content::Blank => wire::cell::Content::Blank(true),
            Content::LiteralText(s) => wire::cell::Content::Text(s),
            Content::Boolean(b) => wire::cell::Content::Boolean(b),
            Content::Decimal(d) => {
                let (lo, mid, hi, negative, scale) = d.parts();
                wire::cell::Content::Number(Decimal128 {
                    lo,
                    mid,
                    hi,
                    negative,
                    scale,
                })
            }
            Content::TextFormula(f) => {
                view.authored_formula = f.original_source.clone();
                wire::cell::Content::Formula(effective.unwrap_or(f.original_source))
            }
            Content::Formula { source, .. } => {
                view.authored_formula = source.clone();
                wire::cell::Content::Formula(effective.unwrap_or(source))
            }
            Content::StructuredFormula(_) => {
                return Err("Experimental geometry frame refused".into());
            }
        });
        match capture.evaluate_values(&[cell.reference]) {
            Ok(value) => match &value.values[0].1 {
                Ok(value) => {
                    if let pivot_types::PivotValue::Decimal(d) = value {
                        view.formatted_number =
                            pivot_workbook::formatting::render_decimal(*d, style)
                                .map_err(|e| e.to_string())?;
                    }
                    set_value(&mut view, value.clone());
                }
                Err(error) => {
                    structured::set_cell_fault(&mut view, error, capture);
                    view.error_code = error_code(error);
                    view.error = readable_error(format!("{error:?}"));
                }
            },
            Err(error) => view.error = readable_error(error),
        }
    } else {
        view.content = Some(wire::cell::Content::Blank(true));
        set_value(&mut view, pivot_types::PivotValue::Blank);
    }
    r.cells.push(view);
    r.styles.push(wire::StyledPosition {
        at: Some(location(position)),
        style: style.encode().to_vec(),
    });
    Ok(())
}
impl Session {
    pub(super) fn handle_geometry(
        &mut self,
        q: &wire::Request,
        r: &mut wire::Response,
        op: wire::request::Operation,
    ) -> Result<(), String> {
        use wire::request::Operation as Op;
        if self.proof.is_some() {
            return Err("Section geometry is unavailable in experimental proofs".into());
        }
        let scope = request_scope(q)?;
        r.worksheet = scope.uuid().to_string();
        r.window_scope = scope.is_window();
        let book = self.book.as_mut().ok_or("create workbook first")?;
        if op == Op::GeometryEdit {
            if q.revision != self.revision {
                r.conflict = true;
                return Err("expected-state conflict".into());
            }
            let commands = q
                .geometry_commands
                .iter()
                .map(|c| {
                    use wire::geometry_command::Action as A;
                    Ok(
                        match A::try_from(c.action).map_err(|_| "unknown geometry command")? {
                            A::AddBreak => Command::AddBreak {
                                horizontal: c.horizontal,
                                before: c.at,
                            },
                            A::RemoveBreak => Command::RemoveBreak {
                                horizontal: c.horizontal,
                                before: c.at,
                            },
                            A::SetProfile => Command::SetProfile {
                                horizontal: c.horizontal,
                                at: c.at,
                                sizes: c
                                    .sizes
                                    .iter()
                                    .map(|s| SizeRun {
                                        first: s.first,
                                        last: s.last,
                                        size: s.size,
                                    })
                                    .collect(),
                            },
                            A::SetSizes => Command::SetSizes {
                                horizontal: c.horizontal,
                                at: c.at,
                                first: c.first,
                                last: c.last,
                                size: c.size,
                                paired: c.paired,
                                shared: c.shared,
                            },
                            A::AcknowledgeLayout => Command::AcknowledgeLayout {
                                stamp: c
                                    .layout_stamp
                                    .as_slice()
                                    .try_into()
                                    .map_err(|_| "invalid layout stamp")?,
                            },
                            A::ResetSizes => Command::ResetSizes,
                            A::ResetAll => Command::ResetAll,
                            _ => return Err("unspecified geometry command"),
                        },
                    )
                })
                .collect::<Result<Vec<_>, &str>>()?;
            let before = book.expected();
            if q.geometry_preview {
                let profiles = book
                    .preview_geometry_profiles(
                        scope,
                        book.expected(),
                        &commands,
                        &Default::default(),
                    )
                    .map_err(|e| e.to_string())?;
                r.geometry_profiles = profiles
                    .into_iter()
                    .map(|p| wire::GeometryProfilePreview {
                        horizontal: p.horizontal,
                        at: p.at,
                        sizes: p
                            .sizes
                            .into_iter()
                            .map(|s| wire::GeometrySizeRun {
                                first: s.first,
                                last: s.last,
                                size: s.size,
                            })
                            .collect(),
                    })
                    .collect();
            } else {
                book.edit_geometry(scope, book.expected(), &commands, &Default::default())
                    .map_err(|e| e.to_string())?;
            }
            if book.expected() != before {
                self.revision = Uuid::new_v4().to_string();
            }
            r.revision = self.revision.clone();
            return Ok(());
        }
        let capture = self.captures.get_mut(&q.capture).ok_or("unknown capture")?;
        if capture.scope != scope || capture.proof.is_some() {
            return Err("geometry capture scope mismatch".into());
        }
        let memory = self.runtime.as_ref().ok_or("missing runtime")?.memory();
        // Use the capture's own pinned visibility and geometry, never a current
        // workbook map against older values. Bound projections retain every row.
        if capture.geometry.is_none() {
            let positional = capture
                .value
                .positional_geometry(scope)
                .map_err(|e| e.to_string())?;
            let current = capture.value.expected() == book.expected();
            let mut filter = wire::Response {
                filter_generation: format!(
                    "geometry:{}:{}",
                    scope.uuid(),
                    capture.value.expected().state
                ),
                ..Default::default()
            };
            let visibility = if positional {
                None
            } else if current {
                if book.filter(scope).map_err(|e| e.to_string())?.is_some() {
                    self.filter.budget_status(&memory)?;
                }
                self.filter.ensure(book, scope, false)?;
                self.filter.describe(book, scope, &mut filter)?;
                if filter.filter_status == 1 {
                    Some(self.filter.rows(book, scope)?)
                } else {
                    None
                }
            } else if let Some(definition) = capture.value.filter_definition() {
                filter.filter_summary = Some(wire::FilterSummary {
                    first: Some(location(definition.rectangle.first)),
                    last: Some(location(definition.rectangle.last)),
                    exclude_header: definition.exclude_header,
                    needs_review: definition.needs_review,
                    columns: definition.conditions.iter().map(|c| c.column).collect(),
                });
                filter.filter_status = if definition.needs_review { 2 } else { 1 };
                if definition.needs_review {
                    None
                } else {
                    match capture.value.filter_visibility(&Default::default()) {
                        Ok(v) => Some(v.rows().to_vec()),
                        Err(error) => {
                            filter.filter_status = 4;
                            filter.filter_error = readable_error(error);
                            None
                        }
                    }
                }
            } else {
                None
            };
            let map =
                match capture
                    .value
                    .geometry_map(scope, visibility.as_deref(), &Default::default())
                {
                    Ok(map) => map,
                    Err(error) if visibility.is_some() => {
                        let reason = readable_error(error);
                        if current {
                            self.filter.suspend_geometry(book, scope, reason)?;
                            self.filter.describe(book, scope, &mut filter)?;
                        } else {
                            filter.filter_status = 2;
                            filter.filter_error = reason;
                            if let Some(summary) = &mut filter.filter_summary {
                                summary.needs_review = true;
                            }
                        }
                        capture
                            .value
                            .geometry_map(scope, None, &Default::default())
                            .map_err(|e| e.to_string())?
                    }
                    Err(error) => return Err(error.to_string()),
                };
            let layout_work = capture
                .value
                .geometry_layout_work(scope)
                .map_err(|e| e.to_string())?;
            if map.construction_work().saturating_add(layout_work)
                > pivot_workbook::geometry::MAX_WORK
            {
                return Err("geometry presentation work limit".into());
            }
            let (layout_stamp, layout_needs_review) = capture
                .value
                .geometry_layout_review(scope, &Default::default())
                .map_err(|e| e.to_string())?;
            capture.geometry = Some(Lease {
                layout_stamp,
                layout_needs_review,
                map,
                viewport: None,
                payload: None,
                emitted_bytes: 1024 * 1024,
                filter,
            });
        }
        let lease = capture.geometry.as_mut().unwrap();
        let map = &lease.map;
        let (rows, columns) = map.dimensions();
        let mut frame = wire::GeometryFrame {
            layout_stamp: lease.layout_stamp.map_or_else(Vec::new, |s| s.to_vec()),
            layout_needs_review: lease.layout_needs_review,
            rows,
            columns,
            filter_summary: lease.filter.filter_summary.clone(),
            filter_status: lease.filter.filter_status,
            filter_error: lease.filter.filter_error.clone(),
            visibility_stamp: lease.filter.filter_generation.clone(),
            positional: capture
                .value
                .positional_geometry(scope)
                .map_err(|e| e.to_string())?,
            horizontal_starts: map.horizontal_starts().to_vec(),
            vertical_starts: map.vertical_starts().to_vec(),
            ..Default::default()
        };
        if op == Op::GeometrySelection {
            let mut requested = Rectangle {
                first: pos(q.first.as_ref())?,
                last: pos(q.last.as_ref())?,
            };
            if q.geometry_paste {
                if q.revision != self.revision || capture.value.expected() != book.expected() {
                    return Err("Paste footprint requires the current capture".into());
                }
                visible::validate_native_kind(q)?;
                if q.geometry_visible_paste {
                    if !matches!(lease.filter.filter_status, 0 | 1) {
                        return Err("Visible paste requires a qualified filter view".into());
                    }
                    let (height, width) = book
                        .clipboard_shape(book.expected(), &q.clipboard, q.native_clipboard)
                        .map_err(|e| e.to_string())?;
                    if height == 0 || width == 0 || height as u64 * width as u64 > 4096 {
                        return Err("visible paste footprint limit".into());
                    }
                    if requested.first == requested.last {
                        if !map.row_is_visible(requested.first.row) {
                            return Err("Visible paste anchor is hidden".into());
                        }
                        let rows: Vec<_> = (requested.first.row..=map.dimensions().0)
                            .filter(|r| map.row_is_visible(*r))
                            .take(height as usize)
                            .collect();
                        if rows.len() != height as usize {
                            return Err("Visible paste exceeds scope extent".into());
                        }
                        requested.last = Position::new(
                            *rows.last().unwrap(),
                            requested
                                .first
                                .column
                                .checked_add(width - 1)
                                .ok_or("paste overflow")?,
                        );
                    }
                } else {
                    requested = book
                        .paste_extent(
                            scope,
                            book.expected(),
                            requested,
                            &q.clipboard,
                            q.native_clipboard,
                        )
                        .map_err(|e| e.to_string())?;
                }
            }
            let selection = map
                .section_closure(requested)
                .map_err(|e| format!("{e:?}"))?;
            frame.requested_first = Some(location(selection.requested.first));
            frame.requested_last = Some(location(selection.requested.last));
            frame.effective_first = Some(location(selection.effective.first));
            frame.effective_last = Some(location(selection.effective.last));
            frame.enlarged = selection.enlarged;
            frame.sections = selection.sections;
        } else {
            frame.visible_mask = vec![0; 8192];
            let (data_first, data_last) = frame
                .filter_summary
                .as_ref()
                .and_then(|s| {
                    Some((
                        s.first.as_ref()?.row + u32::from(s.exclude_header),
                        s.last.as_ref()?.row.min(rows),
                    ))
                })
                .unwrap_or((1, 0));
            frame.filter_total_data_rows = if data_first <= data_last {
                data_last - data_first + 1
            } else {
                0
            };
            for row in 1..=rows {
                if map.row_is_visible(row) {
                    if row >= data_first && row <= data_last {
                        frame.filter_shown_data_rows += 1;
                    }
                    frame.visible_mask[(row as usize - 1) / 8] |= 1 << ((row - 1) % 8);
                }
            }
            let v = q
                .geometry_viewport
                .as_ref()
                .ok_or("missing document viewport")?;
            let mut view = CellRect {
                left: v.left,
                top: v.top,
                right: v.right,
                bottom: v.bottom,
            };
            if let Some(origin) = &q.geometry_origin
                && rows > 0
                && columns > 0
                && map.tile_count() > 0
            {
                frame.scroll_anchor = map
                    .scroll_anchor(Position::new(origin.row, origin.column))
                    .map(location);
                let origin = map
                    .scroll_origin(Position::new(origin.row, origin.column))
                    .ok_or("No scroll anchor in this capture")?;
                view.left = view
                    .left
                    .checked_add(origin.0)
                    .ok_or("geometry coordinate overflow")?;
                view.right = view
                    .right
                    .checked_add(origin.0)
                    .ok_or("geometry coordinate overflow")?;
                view.top = view
                    .top
                    .checked_add(origin.1)
                    .ok_or("geometry coordinate overflow")?;
                view.bottom = view
                    .bottom
                    .checked_add(origin.1)
                    .ok_or("geometry coordinate overflow")?;
            }
            frame.viewport = Some(wire::GeometryRect {
                left: view.left,
                top: view.top,
                right: view.right,
                bottom: view.bottom,
            });
            if q.geometry_after == 0 {
                let active = q
                    .geometry_active
                    .as_ref()
                    .or(q.geometry_origin.as_ref())
                    .map(|p| Position::new(p.row, p.column))
                    .unwrap_or(Position::new(1, 1));
                let headers = map
                    .headers(active, view, &memory, &Default::default())
                    .map_err(|e| format!("{e:?}"))?;
                frame.headers = headers
                    .items
                    .iter()
                    .map(|h| wire::GeometryHeader {
                        column: h.column,
                        index: h.index,
                        first: h.first,
                        last: h.last,
                        section_first: h.section_first,
                        section_last: h.section_last,
                    })
                    .collect();
                if active.row > 0
                    && active.row <= rows
                    && active.column > 0
                    && active.column <= columns
                {
                    let mut selected = wire::Response::default();
                    emit_cell(&capture.value, scope, active, &mut selected)?;
                    frame.active_cell = selected.cells.pop();
                    frame.active_style = selected.styles.pop().map(|s| s.style).unwrap_or_default();
                }
            }
            if q.geometry_after == 0 {
                lease.emitted_bytes = lease
                    .emitted_bytes
                    .checked_add(
                        frame
                            .headers
                            .iter()
                            .map(|h| h.encoded_len() * 4 + 128)
                            .sum::<usize>()
                            + frame
                                .active_cell
                                .as_ref()
                                .map_or(0, |c| c.encoded_len() * 4 + 512),
                    )
                    .filter(|n| *n <= 8 * 1024 * 1024)
                    .ok_or("viewport metadata budget exhausted")?;
            }
            if lease.viewport.is_some_and(|old| old != view) {
                return Err("A changed viewport requires a fresh capture".into());
            }
            if lease.payload.is_none() {
                lease.payload = Some(
                    memory
                        .reserve(8 * 1024 * 1024)
                        .map_err(|_| "viewport payload admission")?,
                );
                lease.viewport = Some(view);
            }
            let page = map
                .viewport_page(
                    view,
                    q.geometry_after as usize,
                    &memory,
                    &Default::default(),
                )
                .map_err(|e| match e {
                    pivot_workbook::geometry::MapError::Limit => {
                        "Viewport exceeds presentation limits; zoom in or use a smaller viewport."
                            .to_owned()
                    }
                    _ => format!("{e:?}"),
                })?;
            frame.total = page.total as u32;
            for cell in &page.cells {
                let previous_cells = r.cells.len();
                let previous_styles = r.styles.len();
                emit_cell(&capture.value, scope, cell.position, r)?;
                let v = cell.rectangle;
                frame.cells.push(wire::GeometryCell {
                    at: Some(location(cell.position)),
                    rectangle: Some(wire::GeometryRect {
                        left: v.left,
                        top: v.top,
                        right: v.right,
                        bottom: v.bottom,
                    }),
                });
                if r.encoded_len() + frame.encoded_len() > MAX_FRAME / 4 {
                    r.cells.truncate(previous_cells);
                    r.styles.truncate(previous_styles);
                    frame.cells.pop();
                    if frame.cells.is_empty() {
                        return Err("A geometry cell exceeds the existing frame budget".into());
                    }
                    break;
                }
            }
            let next = q.geometry_after as usize + frame.cells.len();
            frame.has_next = next < page.total;
            frame.next = if frame.has_next { next as u32 } else { 0 };
            lease.emitted_bytes = lease
                .emitted_bytes
                .checked_add(
                    (r.cells.iter().map(Message::encoded_len).sum::<usize>()
                        + r.styles.iter().map(Message::encoded_len).sum::<usize>()
                        + frame.cells.iter().map(Message::encoded_len).sum::<usize>())
                    .saturating_mul(4)
                    .saturating_add(frame.cells.len() * 512),
                )
                .filter(|n| *n <= 8 * 1024 * 1024)
                .ok_or("viewport payload budget exhausted")?;
        }
        r.capture_identity = q.capture.clone();
        r.generation = q.capture.clone();
        r.capture_state = capture.computational_state.clone();
        r.revision = capture.revision.clone();
        r.geometry = Some(frame);
        Ok(())
    }
}
