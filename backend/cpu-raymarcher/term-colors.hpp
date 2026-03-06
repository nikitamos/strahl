#pragma once

#include <ostream>

// Note: this macro is undef'ed at the end of file
#define ENABLE_COLOR

namespace LR6 {
enum Color {
  kBlack = 0,
  kRed = 1,
  kGreen = 2,
  kYellow = 3,
  kBlue = 4,
  kPurple = 5,
  kCyan = 6,
  kWhite = 7,
  kDefault = 9
};
enum Style { kNormal = 0, kDim = 2, kBold = 1 };

struct EscapeSequence {
  Color color;
  Style style;
  constexpr explicit EscapeSequence(Color color, Style style = kNormal)
      : color(color), style(style) {}
  constexpr explicit EscapeSequence(Style style)
      : color(kDefault), style(style) {}
};

constexpr EscapeSequence operator+(Color color, Style style) {
  return EscapeSequence(color, style);
}
constexpr EscapeSequence operator+(Style style, Color color) {
  return EscapeSequence(color, style);
}
static const constinit EscapeSequence kErrorFormat = kRed + kBold;
static const constinit EscapeSequence kReset(kDefault + kNormal);
static const constinit EscapeSequence kWarningFormat = kYellow + kDim;

std::ostream& operator<<(std::ostream& stream, EscapeSequence seq);
std::ostream& operator<<(std::ostream& stream, Color color);
std::ostream& operator<<(std::ostream& stream, Style style);
}

inline std::ostream& LR6::operator<<(std::ostream& stream, LR6::EscapeSequence seq) {
#ifdef ENABLE_COLOR
  constexpr const size_t kStyleIndex = 2;
  constexpr const size_t kColorIndex = 5;
  char escape[8] = "\e[X;3Xm";
  escape[kColorIndex] = '0' + seq.color;
  escape[kStyleIndex] = '0' + seq.style;
  return stream << escape;
#else
  return stream;
#endif
}

inline std::ostream& LR6::operator<<(std::ostream& stream, LR6::Color color) {
#ifdef ENABLE_COLOR
  return stream << LR6::EscapeSequence(color);
#else
  return stream;
#endif
}
inline std::ostream& LR6::operator<<(std::ostream& stream, LR6::Style style) {
#ifdef ENABLE_COLOR
  return stream << EscapeSequence(style);
#else
  return stream;
#endif
}

#undef ENABLE_COLOR