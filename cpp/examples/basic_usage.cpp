#include <iostream>
#include <vector>
#include <cassert>
#include "msg/msg.hpp"

int main() {
    // Create message
    Msg original;
    original.body = "Hello from C++!";
    original.fromId = "cpp_example";
    original.id = "example_001";
    original.toIds = {"user1", "user2"};
    original.type = "example";

    std::cout << "Original message body: " << original.body << std::endl;

    // Serialize
    auto bytes = original.encode();
    std::cout << "Serialized size: " << bytes.size() << " bytes" << std::endl;

    // Deserialize
    Msg decoded;
    decoded.decode(bytes);

    // Verify
    assert(decoded.body == original.body);
    assert(decoded.fromId == original.fromId);
    assert(decoded.id == original.id);
    assert(decoded.toIds == original.toIds);
    assert(decoded.type == original.type);

    std::cout << "✅ C++ serialization test passed!" << std::endl;
    return 0;
}
