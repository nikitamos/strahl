#include <memory>

template <typename T>
using rc = std::shared_ptr<T>;

template <typename T, typename... Args>
rc<T> rc_new(Args&&... args) {
  return std::make_shared<T>(args...);
}

namespace obj_alg {
template <typename TExpr>
struct ExprAlgebra {
  virtual rc<TExpr> lit(int a) = 0;
  virtual rc<TExpr> add(rc<TExpr> l, rc<TExpr> r) = 0;
};

struct Expr {
  virtual ~Expr() {}
};
struct Lit : public Expr {
  Lit(int x) : l(x) {}
  int l;
};
struct Add : public Expr {
  Add(rc<Expr> l, rc<Expr> r) : l(l), r(r) {}
  rc<Expr> l, r;
};

struct IntFactory : public ExprAlgebra<Expr> {
  rc<obj_alg::Expr> lit(int a) override { return rc_new<Lit>(a); }
  rc<obj_alg::Expr> add(rc<Expr> l, rc<obj_alg::Expr> r) override {
    return rc_new<Add>(l, r);
  }
};

template <typename T>
rc<T> DoStuff(ExprAlgebra<T> *a) {
  return a->Add(a->lit(12), a->lit(43));
}

struct AbstractHypotheticalBackend {
  virtual void Create() = 0;
};

template <typename T>
struct HypotheticalBackendWrapper {
    
};
struct VulkanHypotheticalBackend {
  
};
struct HypotheticalBackend {
  
};



} // namespace obj_alg

using obj_alg::Expr;

int main() {}
