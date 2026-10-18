using Google.Protobuf;
using Pivot.Protocol.V1;

var directory = args[0];
var request = Request.Parser.ParseFrom(File.ReadAllBytes(Path.Combine(directory, "rust-request.bin")));
if (request.RequestId != "request_fixture" || request.InvokeModel.Inputs[0].Value.Decimal.Scale != 28)
    throw new Exception("Rust request mismatch");
File.WriteAllBytes(Path.Combine(directory, "rust-request-roundtrip.bin"), request.ToByteArray());
for (var scale = 0; scale <= 28; scale++)
foreach (var negative in new[] { false, true })
{
    var wire = Decimal128.Parser.ParseFrom(File.ReadAllBytes(Path.Combine(directory,
        $"rust-decimal-{scale}-{negative.ToString().ToLowerInvariant()}.bin")));
    var value = new decimal(unchecked((int)wire.Lo), unchecked((int)wire.Mid), unchecked((int)wire.Hi), wire.Negative, checked((byte)wire.Scale));
    var bits = decimal.GetBits(value);
    if ((uint)bits[0] != wire.Lo || (uint)bits[1] != wire.Mid || (uint)bits[2] != wire.Hi
        || ((bits[3] >> 16) & 255) != scale || (bits[3] < 0) != negative)
        throw new Exception("Decimal representation mismatch");
}
var number = -123.45m;
var parts = decimal.GetBits(number);
var decimalValue = new Value { Decimal = new Decimal128 {
    Lo = unchecked((uint)parts[0]), Mid = unchecked((uint)parts[1]), Hi = unchecked((uint)parts[2]),
    Scale = (uint)((parts[3] >> 16) & 255), Negative = parts[3] < 0
} };
request.InvokeModel.Inputs[0].Value = decimalValue;
File.WriteAllBytes(Path.Combine(directory, "csharp-request.bin"), request.ToByteArray());
var result = new InvocationResult { InvocationId = "inv_fixture", Model = request.InvokeModel.Model.Clone() };
result.Outputs.Add(new OutputResult { PortId = "net", Value = decimalValue });
result.Outputs.Add(new OutputResult { PortId = "margin", Error = new CalculationError { Code = CalculationErrorCode.DivisionByZero, Message = "division by zero" } });
File.WriteAllBytes(Path.Combine(directory, "csharp-result.bin"), result.ToByteArray());
// Unknown future oneof is retained as unknown data, never interpreted as decimal zero.
var future = Value.Parser.ParseFrom(new byte[] { 0x1a, 0 });
if (future.KindCase != Value.KindOneofCase.None) throw new Exception("Unknown value incorrectly accepted");
Console.WriteLine("C# protocol fixtures: PASS (58 decimal layouts, requests, values, errors)");
var descriptors = new Dictionary<string, Google.Protobuf.Reflection.MessageDescriptor>();
var visited = new HashSet<string>();
void Load(Google.Protobuf.Reflection.FileDescriptor file)
{
    if (!visited.Add(file.Name)) return;
    foreach (var type in file.MessageTypes) descriptors[type.FullName] = type;
    foreach (var dependency in file.Dependencies) Load(dependency);
}
Load(Pivot.Protocol.Authoring.V1.AuthoringReflection.Descriptor);
foreach (var line in File.ReadAllLines(Path.Combine(directory, "authoring-manifest.txt")))
{
    var fields = line.Split('|');
    var parsed = descriptors[fields[0].TrimStart('.')].Parser.ParseFrom(File.ReadAllBytes(Path.Combine(directory, fields[1])));
    File.WriteAllBytes(Path.Combine(directory, "csharp-" + fields[1]), parsed.ToByteArray());
}
var patch = new Pivot.Protocol.Authoring.V1.SetInputPort {
    Port = new Pivot.Protocol.Authoring.V1.PortReference { PortId = "01234567-89ab-4cde-8fab-0123456789ab" },
    Requirement = new Pivot.Protocol.Authoring.V1.Requirement { Required = new Pivot.Protocol.Authoring.V1.Required() }
};
if (patch.Node != null || Pivot.Protocol.Authoring.V1.SetInputPort.Parser.ParseFrom(patch.ToByteArray()).Node != null)
    throw new Exception("Absent port update lost presence");
File.WriteAllBytes(Path.Combine(directory, "csharp-patch.bin"), patch.ToByteArray());
