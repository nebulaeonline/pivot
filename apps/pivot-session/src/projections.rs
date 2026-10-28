use super::*;
use pivot_workbook::projection::{self, ProjectionId};
impl Session {
    // All publication, workbook edits and composite capture run under &mut Session
    // on the single request owner. Detached calculation owns immutable captures only.
    pub(super) fn handle_proof(
        &mut self,
        q: &wire::Request,
        r: &mut wire::Response,
        op: wire::request::Operation,
    ) -> Result<(), String> {
        use wire::request::Operation as Op;
        if q.revision != self.revision {
            r.conflict = true;
            return Err("expected-state conflict".into());
        }
        if op == Op::ProofStart {
            if self.proof.is_some() || !self.captures.is_empty() {
                return Err("Proof already active or captures retained".into());
            }
            self.book.as_ref().ok_or("create workbook first")?;
            let mut proof = projection::State::new(self.runtime.as_ref().ok_or("closed")?)
                .map_err(|e| e.to_string())?;
            proof
                .publish(
                    0,
                    proof.schema,
                    &projection::State::fixture(3).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
            proof.changed = false;
            self.proof = Some(proof);
            return Ok(());
        }
        if op == Op::ProofEvaluate {
            let captured = self
                .captures
                .get(&q.capture)
                .and_then(|c| c.proof.as_ref())
                .ok_or("unknown structured capture")?;
            let mut value = wire::ProofEvaluation {
                generation: captured
                    .available_generation()
                    .map(|id| id.0.to_string())
                    .unwrap_or_default(),
                epoch: captured.epoch,
                ..Default::default()
            };
            match captured.evaluate() {
                Ok((total, taxed)) => {
                    value.total = Some(decimal(total));
                    value.taxed = Some(decimal(taxed));
                }
                Err(e) => value.error = e.to_string(),
            }
            if captured.available_generation().is_some() && !q.proof_key.is_empty() {
                match captured.generation.record(&q.proof_key) {
                    Ok(record) => value.description = record.description.clone(),
                    Err(e) => value.error = e.to_string(),
                }
            }
            r.proof_value = Some(value);
            return Ok(());
        }
        let proof = self
            .proof
            .as_mut()
            .ok_or("experimental proof is not active")?;
        match op {
            Op::ProofPublish => {
                let mut rows = projection::State::fixture(q.proof_count as usize)
                    .map_err(|e| e.to_string())?;
                if !q.proof_key.is_empty() {
                    rows.retain(|r| r.key != q.proof_key)
                }
                proof
                    .publish(q.proof_epoch, proof.schema, &rows)
                    .map_err(|e| e.to_string())?;
            }
            Op::ProofLayout => {
                let id = ProjectionId(
                    Uuid::parse_str(&q.proof_projection)
                        .map_err(|_| "invalid projection identity")?,
                );
                proof
                    .layout(q.proof_presentation, id, q.proof_after_row, q.proof_reverse)
                    .map_err(|e| e.to_string())?;
            }
            Op::ProofRead => {
                let first = pos(q.first.as_ref())?;
                let last = pos(q.last.as_ref())?;
                if first.column == 0
                    || last.column > 1024
                    || last.column < first.column
                    || last.column - first.column + 1 > 10
                    || last.row < first.row
                {
                    return Err("invalid projection viewport".into());
                }
                let frame = proof
                    .view(first.row, last.row - first.row + 1)
                    .map_err(|e| e.to_string())?;
                let mut view = wire::ProofFrame {
                    producer: proof.producer.0.to_string(),
                    generation: frame
                        .generation
                        .map(|g| g.0.to_string())
                        .unwrap_or_default(),
                    epoch: frame.epoch,
                    presentation: frame.presentation,
                    record_count: frame.count,
                    error: frame.failure.map(|e| e.to_string()).unwrap_or_default(),
                    schema: proof.schema.0.to_string(),
                    field_ids: proof.fields.iter().map(|f| f.0.to_string()).collect(),
                    output: proof.output.0.to_string(),
                    record_keys: proof.record_keys(),
                    first_visual_row: first.row,
                    ..Default::default()
                };
                view.placements = frame
                    .placements
                    .iter()
                    .map(|p| wire::ProofPlacement {
                        id: p.id.0.to_string(),
                        after_row: p.after_row,
                        reverse: p.reverse,
                    })
                    .collect();
                view.rows = frame
                    .rows
                    .iter()
                    .map(|row| wire::ProofRow {
                        kind: row.kind,
                        ordinary_row: row.ordinary,
                        projection: row.projection.map(|p| p.0.to_string()).unwrap_or_default(),
                        key: row.key.clone(),
                        ordinal: row.ordinal,
                        fields: row.fields.clone(),
                    })
                    .collect();
                let ordinary: Vec<_> = frame
                    .rows
                    .iter()
                    .filter(|row| row.kind == 0)
                    .map(|row| row.ordinary)
                    .collect();
                r.capture_state = self.computational_state.clone();
                r.capture_identity = Uuid::new_v4().to_string();
                r.generation = r.capture_identity.clone();
                r.captured_source_epoch = proof.epoch;
                r.captured_source_generation = proof
                    .current_generation()
                    .map(|id| id.0.to_string())
                    .unwrap_or_default();
                if let (Some(min), Some(max)) = (ordinary.first(), ordinary.last()) {
                    // Ordinary rows visible around a band remain a bounded logical span.
                    let nested = wire::Request {
                        version: 5,
                        worksheet: q.worksheet.clone(),
                        revision: q.revision.clone(),
                        operation: Op::Read as i32,
                        first: Some(wire::Position {
                            row: *min,
                            column: first.column,
                        }),
                        last: Some(wire::Position {
                            row: *max,
                            column: last.column,
                        }),
                        ..Default::default()
                    };
                    self.handle(&nested, r)?;
                }
                r.proof_view = Some(view);
                self.proof_frame = Some(frame);
            }
            _ => return Err("unsupported proof operation".into()),
        }
        Ok(())
    }
}
