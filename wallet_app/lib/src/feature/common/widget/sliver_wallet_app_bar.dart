import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/num_extensions.dart';
import 'button/icon/back_icon_button.dart';
import 'stepper_indicator.dart';

/// The space taken up by the stepper indicator (when visible).
const kStepIndicatorHeight = 4.0;

class SliverWalletAppBar extends StatefulWidget {
  final String title;
  final double? progress;
  final List<Widget>? actions;
  final Widget? leading;

  const SliverWalletAppBar({
    required this.title,
    this.leading,
    this.progress,
    this.actions,
    super.key,
  });

  @override
  State<SliverWalletAppBar> createState() => _SliverWalletAppBarState();
}

class _SliverWalletAppBarState extends State<SliverWalletAppBar> {
  /// Exposes the collapsed ratio (from the expanded to collapsed toolbar height), useful for animating widgets.
  ValueNotifier<double> collapsedRatio = ValueNotifier(1.0);

  /// Cached height of the title, used to avoid having to re-calculate on every rebuilt of this widget.
  /// Automatically invalidated when the title changes.
  double? _textHeightCache;

  /// Top padding (insets), used as a small optimization to avoid resolving on every frame.
  late double _topPadding;

  /// Padding used to indent the expanded title, also used when calculating textHeight.
  final _expandedTitlePadding = const EdgeInsets.only(bottom: 14, left: 16, top: 0, right: 16);

  /// The point in the expand/collapse animation where the expanded title is transformed into the appbar title.
  final _titleCrossFadePoint = 0.7;

  double get toolbarHeight => widget.progress == null ? kToolbarHeight : kToolbarHeight + kStepIndicatorHeight;

  double get expandedHeight => toolbarHeight + 32 + (_textHeightCache ?? _calculateTextHeight(widget.title));

  @override
  Widget build(BuildContext context) {
    _topPadding = context.mediaQuery.padding.top;
    final canPop = Navigator.of(context).canPop();
    return SliverAppBar(
      pinned: true,
      title: ValueListenableBuilder<double>(
        builder: (context, collapsedRatio, child) {
          return Opacity(
            opacity: 1.0 - collapsedRatio.normalize(0.0, _titleCrossFadePoint),
            child: child,
          );
        },
        valueListenable: collapsedRatio,
        child: Text(widget.title),
      ),
      leading: widget.leading ?? (canPop ? const BackIconButton() : null),
      titleSpacing: widget.leading == null && !canPop ? null : 0.0,
      actions: widget.actions,
      collapsedHeight: toolbarHeight,
      expandedHeight: expandedHeight,
      flexibleSpace: LayoutBuilder(
        builder: (context, constraints) {
          /// Calculate the flexibleSpace's collapsed ratio, used to animate titles
          final minHeight = _topPadding + toolbarHeight;
          final maxHeight = _topPadding + expandedHeight;
          final collapsedRatio = constraints.maxHeight.normalize(minHeight, maxHeight).toDouble();

          /// Notify the [collapsedRatio] ValueNotifier so the SliverAppBar.title can be animated.
          WidgetsBinding.instance.addPostFrameCallback((duration) => this.collapsedRatio.value = collapsedRatio);

          /// Calculate the opacity of the expanded title
          final expandedTextOpacity = collapsedRatio.normalize(_titleCrossFadePoint, 1.0).toDouble();

          /// Render the flexible space, which includes the (optional) progress bar and expanded title
          return Stack(
            children: [
              if (widget.progress != null) _buildPositionedProgressBar(),
              FlexibleSpaceBar(
                expandedTitleScale: 1.0,
                centerTitle: false,
                titlePadding: _expandedTitlePadding,
                title: Opacity(
                  opacity: expandedTextOpacity,
                  child: SafeArea(
                    top: false,
                    bottom: false,
                    child: Text(
                      widget.title,
                      style: context.textTheme.displayMedium,
                    ),
                  ),
                ),
              ),
            ],
          );
        },
      ),
    );
  }

  @override
  void didUpdateWidget(covariant SliverWalletAppBar oldWidget) {
    super.didUpdateWidget(oldWidget);
    _textHeightCache = null;
  }

  Widget _buildPositionedProgressBar() {
    return Positioned(
      top: _topPadding + toolbarHeight - kStepIndicatorHeight,
      left: 0,
      right: 0,
      child: SafeArea(
        top: false,
        bottom: false,
        child: StepperIndicator(
          progress: widget.progress ?? 0.0,
        ),
      ),
    );
  }

  double _calculateTextHeight(String text) {
    TextPainter tp = TextPainter(
      text: TextSpan(text: text, style: context.textTheme.displayMedium),
      textDirection: TextDirection.ltr,
      textScaler: MediaQuery.of(context).textScaler,
    );
    tp.layout(maxWidth: MediaQuery.of(context).size.width - _expandedTitlePadding.horizontal);
    return _textHeightCache = tp.height;
  }
}
