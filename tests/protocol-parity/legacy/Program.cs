using Google.Protobuf;
using Pivot.Protocol.V1;
File.WriteAllBytes(Path.Combine(args[0], "legacy-descriptor.bin"), ProtocolReflection.Descriptor.ToProto().ToByteArray());

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
var hostResponse = Path.Combine(directory, "new-host-response.bin");
if (File.Exists(hostResponse))
{
    var response = Response.Parser.ParseFrom(File.ReadAllBytes(hostResponse));
    var value = response.Invocation.Outputs[0].Value.Decimal;
    if (response.Version.Minor != 0 || value.Lo != 12445 || value.Scale != 2 || !value.Negative)
        throw new Exception("Frozen v1.0 codec/current-host mismatch");
    Console.WriteLine("Frozen v1.0 C# codec/current host: PASS");
}
