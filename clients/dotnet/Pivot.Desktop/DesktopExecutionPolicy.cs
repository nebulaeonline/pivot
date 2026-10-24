using System;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Security.Cryptography;
using System.Text;

namespace Pivot.Desktop;

// Shared loopback protocol with scripts/execution_policy.py. A live permit is
// issued only after interactive consent; no unattended isolation override exists.
// This protects cooperative runners, not against hostile code under the same user.
internal static class DesktopExecutionPolicy
{
    internal static bool AutomationRequested =>
        Environment.GetEnvironmentVariable("PIVOT_UI_SMOKE_REPORT") is not null ||
        Environment.GetEnvironmentVariable("PIVOT_UI_PEER_REPORT") is not null;

    internal static bool Allowed()
    {
        if (!AutomationRequested) return true; // Ordinary manual application launch.
        try {
            string[] parts = (Environment.GetEnvironmentVariable("PIVOT_DESKTOP_PERMIT") ?? "").Split(':');
            if (parts.Length != 2 || !int.TryParse(parts[0], out int port) || port < 1 || port > 65535 || parts[1].Length != 64) return false;
            byte[] secret = Convert.FromHexString(parts[1]);
            if (secret.Length != 32) return false;
            using var client = new TcpClient(AddressFamily.InterNetwork);
            client.ConnectAsync(IPAddress.Loopback, port).WaitAsync(TimeSpan.FromSeconds(2)).GetAwaiter().GetResult();
            client.ReceiveTimeout = 2000; client.SendTimeout = 2000;
            using var stream = client.GetStream();
            string nonce = Convert.ToHexString(RandomNumberGenerator.GetBytes(32)).ToLowerInvariant();
            byte[] request = Encoding.ASCII.GetBytes(parts[1] + "\n" + nonce + "\n");
            stream.Write(request);
            // Bounded read; fail closed rather than allocating for an arbitrary peer.
            byte[] response = new byte[65];
            int length = 0;
            var deadline = System.Diagnostics.Stopwatch.StartNew();
            while (length < response.Length) {
                if (deadline.Elapsed > TimeSpan.FromSeconds(3)) return false;
                int value = stream.ReadByte();
                if (value < 0) return false;
                response[length++] = (byte)value;
                if (value == '\n') break;
            }
            if (length != 65 || response[64] != '\n') return false;
            byte[] actual = Convert.FromHexString(Encoding.ASCII.GetString(response, 0, 64));
            byte[] expected = HMACSHA256.HashData(secret, Encoding.ASCII.GetBytes(nonce));
            return CryptographicOperations.FixedTimeEquals(actual, expected);
        } catch (Exception error) when (error is IOException or SocketException or FormatException or TimeoutException or ArgumentException) {
            return false;
        }
    }

    internal static bool CheckBeforeStartup()
    {
        if (Allowed()) return true;
        Console.Error.WriteLine("{\"status\":\"Blocked\",\"reason\":\"Desktop self-check requires a live consent permit. Run python scripts/ci.py --desktop or tests/application-session/ui_smoke.py interactively. Unattended isolation is not configured.\"}");
        return false;
    }
}
