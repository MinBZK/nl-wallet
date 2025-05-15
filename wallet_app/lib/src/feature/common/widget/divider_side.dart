enum DividerSide { none, top, bottom, both }

extension DividerSideExtension on DividerSide {
  bool get top => this == DividerSide.top || this == DividerSide.both;

  bool get bottom => this == DividerSide.bottom || this == DividerSide.both;
}
