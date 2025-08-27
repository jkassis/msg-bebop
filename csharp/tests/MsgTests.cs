using Xunit;

namespace Msg.Bebop.Tests
{
    public class MsgTests
    {
        [Fact]
        public void TestBasicSerialization()
        {
            var original = new Msg
            {
                Body = "Hello from C#!",
                FromId = "csharp_test",
                Id = "test_001",
                ToIds = new[] { "user1", "user2" },
                Type = "test"
            };

            // Serialize
            var bytes = original.Encode();
            Assert.True(bytes.Length > 0);

            // Deserialize
            var decoded = Msg.Decode(bytes);

            // Verify
            Assert.Equal(original.Body, decoded.Body);
            Assert.Equal(original.FromId, decoded.FromId);
            Assert.Equal(original.Id, decoded.Id);
            Assert.Equal(original.ToIds, decoded.ToIds);
            Assert.Equal(original.Type, decoded.Type);
        }
    }
}
