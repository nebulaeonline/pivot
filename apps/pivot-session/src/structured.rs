use super::*;
use pivot_workbook::structured::{Occurrence, OutputBinding, SourceFault};
pub(super) fn prepare_capture(
    proof: Option<&pivot_workbook::projection::State>,
    capture: &mut pivot_workbook::CalculationCapture,
) -> Result<(), String> {
    if let Some(proof) = proof {
        if capture
            .structured_binding()
            .is_some_and(|b| b.producer != proof.producer.0)
        {
            return capture
                .prepare_structured_failure(SourceFault::UnavailableResult)
                .map_err(|e| e.to_string());
        }
        match proof.current_total() {
            Ok(value) => capture
                .prepare_structured_context(pivot_core::captured::ScalarValue::Decimal(value), &[]),
            Err(pivot_workbook::projection::Error::Numeric) => {
                capture.prepare_structured_failure(SourceFault::NumericOverflow)
            }
            Err(_) => capture.prepare_structured_failure(SourceFault::UnavailableResult),
        }
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn encode(o: Occurrence) -> wire::TypedOccurrence {
    wire::TypedOccurrence {
        ordinal: o.ordinal,
        start_byte: o.span.start as u32,
        end_byte: o.span.end as u32,
        document: o.binding.document.to_string(),
        producer: o.binding.producer.to_string(),
        output: o.binding.output.to_string(),
        expected_type: 1,
    }
}
fn decode(o: &wire::TypedOccurrence) -> Result<Occurrence, String> {
    if o.expected_type != 1 {
        return Err("structured output type mismatch".into());
    }
    Ok(Occurrence {
        ordinal: o.ordinal,
        span: o.start_byte as usize..o.end_byte as usize,
        binding: OutputBinding {
            document: Uuid::parse_str(&o.document).map_err(|_| "invalid document identity")?,
            producer: Uuid::parse_str(&o.producer).map_err(|_| "invalid producer identity")?,
            output: Uuid::parse_str(&o.output).map_err(|_| "invalid output identity")?,
        },
    })
}
impl Session {
    pub(super) fn handle_structured(
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
        if op == Op::BindingStart {
            if self.proof.is_some() {
                return Err("start bindings in a normal workbook".into());
            }
            self.handle_proof(q, r, Op::ProofStart)?;
            let enabled = self
                .proof
                .as_ref()
                .ok_or("missing proof")?
                .enable_binding_book(self.book.as_mut().ok_or("no workbook")?);
            if let Err(e) = enabled {
                self.proof.take();
                return Err(e.to_string());
            }
            return Ok(());
        }
        let book = self.book.as_mut().ok_or("no workbook")?;
        if !book.is_structured_proof() {
            return Err("binding proof is not enabled".into());
        }
        if q.formula_source.len() > 4096 || q.occurrences.len() > 16 {
            return Err("binding source admission".into());
        }
        let before = book.expected();
        match op {
            Op::ResolveFormula => {
                r.occurrences = book
                    .resolve_structured(&q.formula_source)
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .map(encode)
                    .collect()
            }
            Op::ApplyFormula => {
                let occurrences = q
                    .occurrences
                    .iter()
                    .map(decode)
                    .collect::<Result<Vec<_>, _>>()?;
                book.apply_structured(
                    before,
                    pos(q.first.as_ref())?,
                    &q.formula_source,
                    &occurrences,
                )
                .map_err(|e| e.to_string())?;
            }
            Op::DuplicateBlock => {
                book.duplicate_structured_block(
                    before,
                    pivot_workbook::Rectangle {
                        first: pos(q.first.as_ref())?,
                        last: pos(q.last.as_ref())?,
                    },
                    pivot_workbook::Rectangle {
                        first: pos(q.destination_first.as_ref())?,
                        last: pos(q.destination_last.as_ref())?,
                    },
                )
                .map_err(|e| e.to_string())?;
            }
            Op::RenameOutput => {
                book.rename_structured_output(before, &q.producer_name)
                    .map_err(|e| e.to_string())?;
            }
            Op::RetireOutput | Op::RecreateOutput => {
                book.retire_structured_output(before, op == Op::RecreateOutput)
                    .map_err(|e| e.to_string())?;
                if let Some(binding) = book.current_structured_binding() {
                    self.proof
                        .as_mut()
                        .ok_or("missing proof")?
                        .adopt_output_identity(binding.producer, binding.output);
                }
            }
            Op::RenameProducer => {
                book.rename_structured_producer(before, &q.producer_name)
                    .map_err(|e| e.to_string())?;
            }
            Op::RetireProducer | Op::RecreateProducer => {
                let proof = self.proof.as_ref().ok_or("missing proof")?;
                if proof.epoch == u64::MAX || proof.presentation == u64::MAX {
                    return Err("source epoch exhausted".into());
                }
                book.retire_structured_producer(before, op == Op::RecreateProducer)
                    .map_err(|e| e.to_string())?;
                let proof = self.proof.as_mut().ok_or("missing proof")?;
                proof
                    .publish_unavailable(proof.epoch)
                    .map_err(|e| e.to_string())?;
            }
            Op::PublishUnavailable => {
                let proof = self.proof.as_mut().ok_or("missing producer")?;
                proof
                    .publish_unavailable(q.proof_epoch)
                    .map_err(|e| e.to_string())?;
            }
            _ => return Err("unsupported binding operation".into()),
        }
        if book.expected() != before {
            self.revision = Uuid::new_v4().to_string();
        }
        r.revision = self.revision.clone();
        Ok(())
    }
}

pub(super) fn set_cell_binding(
    cell: &mut wire::Cell,
    capture: &pivot_workbook::CalculationCapture,
) {
    if let Some(b) = capture.structured_binding() {
        cell.source_producer = b.producer.to_string();
        cell.source_output = b.output.to_string();
    }
    if let Some((producer, output)) = capture.structured_labels() {
        cell.source_producer_name = producer.into();
        cell.source_output_name = output.into();
    }
}
pub(super) fn set_cell_fault(
    cell: &mut wire::Cell,
    error: &pivot_workbook::CellError,
    capture: &pivot_workbook::CalculationCapture,
) {
    if let pivot_workbook::CellError::CapturedSourceFailed { fault_id, kind } = error {
        cell.source_fault_id = Uuid::from_bytes(fault_id.0).to_string();
        cell.source_fault_code = format!("{kind:?}");
        set_cell_binding(cell, capture);
    }
}
