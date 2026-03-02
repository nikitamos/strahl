#include <memory>
#include <vector>

#include "node.hpp"

namespace strahl {

class Scene {
  std::vector<std::unique_ptr<Node*>> nodes_;
};

}