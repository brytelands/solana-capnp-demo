@0xd7f46c866337c03c;

using Schema = import "/capnp/schema.capnp";
using Cxx = import "/capnp/c++.capnp";
$Cxx.namespace("person");

struct Person {
  firstname @0 :Text;
  lastname @1 :Text;
}