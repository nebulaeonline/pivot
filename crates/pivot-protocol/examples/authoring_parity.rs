use pivot_protocol::{DESCRIPTORS, foundation};
use prost::Message;
use prost_types::{DescriptorProto, FileDescriptorSet, field_descriptor_proto::Type};
use std::{collections::BTreeMap, env, fs, path::Path};
fn var(mut v: u64, b: &mut Vec<u8>) {
    while v >= 128 {
        b.push((v as u8) | 128);
        v >>= 7;
    }
    b.push(v as u8);
}
fn build(
    name: &str,
    choice: Option<(i32, i32)>,
    messages: &BTreeMap<String, DescriptorProto>,
    enums: &BTreeMap<String, i32>,
) -> Vec<u8> {
    let d = &messages[name];
    let mut bytes = Vec::new();
    for f in &d.field {
        if let Some(index) = f.oneof_index {
            let first = d
                .field
                .iter()
                .find(|x| x.oneof_index == Some(index))
                .unwrap()
                .number();
            let selected = choice
                .filter(|(i, _)| *i == index)
                .map_or(first, |(_, n)| n);
            if f.number() != selected {
                continue;
            }
        }
        let wire = match f.r#type() {
            Type::Fixed32 => 5,
            Type::Double => 1,
            Type::Message | Type::String => 2,
            _ => 0,
        };
        var(((f.number() as u64) << 3) | wire, &mut bytes);
        match f.r#type() {
            Type::Message => {
                let v = build(f.type_name(), None, messages, enums);
                var(v.len() as u64, &mut bytes);
                bytes.extend(v);
            }
            Type::String => {
                let s = if (f.name().ends_with("_id") || f.name().ends_with("_ids"))
                    && !["operation_id", "temporary_id", "request_id"].contains(&f.name())
                {
                    "01234567-89ab-4cde-8fab-0123456789ab"
                } else {
                    "A"
                };
                var(s.len() as u64, &mut bytes);
                bytes.extend(s.as_bytes());
            }
            Type::Fixed32 => bytes.extend(1u32.to_le_bytes()),
            Type::Double => bytes.extend(1f64.to_le_bytes()),
            Type::Enum => var(enums[f.type_name()] as u64, &mut bytes),
            _ => var(1, &mut bytes),
        }
    }
    bytes
}
fn main() {
    let args: Vec<_> = env::args().collect();
    let dir = Path::new(&args[2]);
    fs::create_dir_all(dir).unwrap();
    let mut messages = BTreeMap::new();
    let mut enums = BTreeMap::new();
    for f in FileDescriptorSet::decode(DESCRIPTORS).unwrap().file {
        for d in f.message_type {
            messages.insert(
                format!(".{}.{}", f.package.as_deref().unwrap(), d.name()),
                d,
            );
        }
        for e in f.enum_type {
            enums.insert(
                format!(".{}.{}", f.package.as_deref().unwrap(), e.name()),
                e.value.iter().find(|v| v.number() != 0).unwrap().number(),
            );
        }
    }
    if args[1] == "write" {
        let mut manifest = String::new();
        let mut index = 0;
        for (name, d) in &messages {
            if !name.starts_with(".pivot.authoring.v1.") {
                continue;
            }
            let mut choices = vec![None];
            for f in &d.field {
                if let Some(i) = f.oneof_index {
                    choices.push(Some((i, f.number())));
                }
            }
            for choice in choices {
                let bytes = build(name, choice, &messages, &enums);
                foundation::validate_message(name, &bytes, foundation::Limits::default())
                    .unwrap_or_else(|e| panic!("{name}: {e}"));
                let file = format!("authoring-{index}.bin");
                fs::write(dir.join(&file), bytes).unwrap();
                manifest.push_str(&format!("{name}|{file}\n"));
                index += 1;
            }
        }
        fs::write(dir.join("authoring-manifest.txt"), manifest).unwrap();
        fs::write(dir.join("current-descriptors.bin"), DESCRIPTORS).unwrap();
    } else {
        for line in fs::read_to_string(dir.join("authoring-manifest.txt"))
            .unwrap()
            .lines()
        {
            let (name, file) = line.split_once('|').unwrap();
            let original = fs::read(dir.join(file)).unwrap();
            let returned = fs::read(dir.join(format!("csharp-{file}"))).unwrap();
            foundation::validate_message(name, &returned, foundation::Limits::default()).unwrap();
            assert_eq!(original, returned, "{name}");
        }
        let legacy = prost_types::FileDescriptorProto::decode(
            fs::read(dir.join("legacy-descriptor.bin"))
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        for old in &legacy.enum_type {
            let current = FileDescriptorSet::decode(DESCRIPTORS)
                .unwrap()
                .file
                .into_iter()
                .flat_map(|f| f.enum_type)
                .find(|e| e.name() == old.name())
                .unwrap();
            assert_eq!(old.value, current.value, "legacy enum {}", old.name());
        }
        let patch = a_patch(&fs::read(dir.join("csharp-patch.bin")).unwrap());
        assert!(patch.node.is_none() && patch.requirement.is_some());
        for old in legacy.message_type {
            let new = &messages[&format!(".pivot.v1.{}", old.name())];
            for field in &old.field {
                let new = new
                    .field
                    .iter()
                    .find(|f| f.number() == field.number())
                    .unwrap();
                assert_eq!(
                    (
                        &field.name,
                        field.r#type,
                        field.label,
                        &field.type_name,
                        field.oneof_index,
                        field.proto3_optional
                    ),
                    (
                        &new.name,
                        new.r#type,
                        new.label,
                        &new.type_name,
                        new.oneof_index,
                        new.proto3_optional
                    )
                );
            }
        }
    }
    println!("Authoring and v1.0 descriptor fixtures: PASS");
}
fn a_patch(bytes: &[u8]) -> pivot_protocol::authoring::v1::SetInputPort {
    foundation::validate_message(
        ".pivot.authoring.v1.SetInputPort",
        bytes,
        foundation::Limits::default(),
    )
    .unwrap();
    pivot_protocol::authoring::v1::SetInputPort::decode(bytes).unwrap()
}
