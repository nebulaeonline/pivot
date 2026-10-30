using System.Buffers.Binary;
using Google.Protobuf;
using Pivot.Protocol.Application.V1;
namespace Pivot.Client;

/// <summary>Presentation only. Numeric rendering stays in Rust.</summary>
public sealed record CellStyle(byte Numeric=0,byte Places=2,byte Rounding=1,bool Grouping=false,byte Currency=0,
    bool Bold=false,bool Italic=false,byte Alignment=0,uint? Foreground=null,uint? Background=null)
{
    public byte[] Encode() {
        if(Numeric>3||Places>28||Rounding>6||Currency>2||Alignment>3||Foreground>0xffffff||Background>0xffffff)throw new ArgumentException("Invalid style.");
        byte[] b=[Numeric,Places,Rounding,(byte)(Grouping?1:0),Currency,(byte)(Bold?1:0),(byte)(Italic?1:0),Alignment,0,0,0,0,0,0,0,0,0];
        BinaryPrimitives.WriteUInt32LittleEndian(b.AsSpan(9),Foreground??uint.MaxValue);
        BinaryPrimitives.WriteUInt32LittleEndian(b.AsSpan(13),Background??uint.MaxValue);return b;
    }
    public static CellStyle Decode(ByteString data) {
        var b=data.Span;
        if(b.Length!=17||b[8]!=0||b[3]>1||b[5]>1||b[6]>1)throw new IOException("Invalid style record.");
        uint f=BinaryPrimitives.ReadUInt32LittleEndian(b[9..]),g=BinaryPrimitives.ReadUInt32LittleEndian(b[13..]);
        var s=new CellStyle(b[0],b[1],b[2],b[3]!=0,b[4],b[5]!=0,b[6]!=0,b[7],f==uint.MaxValue?null:f,g==uint.MaxValue?null:g);
        s.Encode();return s;
    }
}
public sealed partial class WorkbookSession {
    public Task<Response> FormatAsync(string expected,Position first,Position last,CellStyle style,uint fields,bool reset=false,CancellationToken cancellation=default) =>
        CallAsync(new Request {Operation=Request.Types.Operation.Format,Revision=expected,First=first.Clone(),Last=last.Clone(),Style=ByteString.CopyFrom(style.Encode()),StyleFields=fields,ResetFormat=reset},cancellation);
    public Task<Response> FormatRegionAsync(string expected,Position first,Position last,CellStyle style,uint fields,bool reset=false,bool inherit=false,bool defaults=false,CancellationToken cancellation=default) =>
        CallAsync(new Request {Operation=Request.Types.Operation.Format,Revision=expected,First=first.Clone(),Last=last.Clone(),Style=ByteString.CopyFrom(style.Encode()),StyleFields=fields,ResetFormat=reset,StyleInherit=inherit,WorksheetDefaults=defaults},cancellation);
    public Task<Response> ReadStyleSummaryAsync(Position first,Position last,CancellationToken cancellation=default) =>
        CallAsync(new Request {Operation=Request.Types.Operation.ReadStyles,First=first.Clone(),Last=last.Clone(),StyleSummary=true},cancellation);
    public Task<Response> ReadStylesAsync(Position first,Position last,CancellationToken cancellation=default) =>
        CallAsync(new Request {Operation=Request.Types.Operation.ReadStyles,First=first.Clone(),Last=last.Clone()},cancellation);
}
