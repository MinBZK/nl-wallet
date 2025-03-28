import 'package:flutter/material.dart';

import '../../../domain/model/flow_progress.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/num_extensions.dart';
import '../../../util/extension/scroll_controller_extensions.dart';
import 'button/icon/back_icon_button.dart';
import 'stepper_indicator.dart';
import 'text/title_text.dart';

/// The space taken up by the stepper indicator (when visible).
const kStepIndicatorHeight = 5.0;

class SliverWalletAppBar extends StatefulWidget {
  final String title;
  final FlowProgress? progress;
  final List<Widget>? actions;
  final Widget? leading;
  final bool automaticallyImplyLeading;

  /// Providing a scroll controller allows the widget to 'speed up' the animation in case the
  /// maxScrollExtent is smaller than the pixels it would normally take to completely transition
  /// to showing the app bar title.
  final ScrollController? scrollController;

  const SliverWalletAppBar({
    required this.title,
    this.leading,
    this.automaticallyImplyLeading = true,
    this.progress,
    this.actions,
    this.scrollController,
    super.key,
  });

  @override
  State<SliverWalletAppBar> createState() => _SliverWalletAppBarState();
}

class _SliverWalletAppBarState extends State<SliverWalletAppBar> {
  /// Exposes the collapsed ratio (from the expanded to collapsed toolbar height), useful for animating widgets.
  ValueNotifier<double> collapsedRatio = ValueNotifier(1);

  /// Cached height of the title, used to avoid having to re-calculate on every rebuilt of this widget.
  /// Automatically invalidated when the title changes.
  double? _textHeightCache;

  /// Padding used to indent the expanded title, also used when calculating textHeight.
  final _expandedTitlePadding = const EdgeInsets.only(bottom: 14, left: 16, top: 0, right: 16);

  /// The point in the expand/collapse animation where the expanded title is transformed into the appbar title.
  final _titleCrossFadePoint = 0.7;

  double get toolbarHeight => widget.progress == null ? kToolbarHeight : kToolbarHeight + kStepIndicatorHeight;

  double get expandedHeight => toolbarHeight + 32 + _getTextHeight(widget.title);

  double get maxScrollExtent => widget.scrollController?.maxScrollExtent(fallback: double.infinity) ?? double.infinity;

  @override
  Widget build(BuildContext context) {
    final titleTextStyle = context.theme.appBarTheme.titleTextStyle;
    final topPadding = context.mediaQuery.padding.top;

    /// Decide if we should show the [WalletBackButton] when no [leading] widget is provided.
    final showBackButton = Navigator.of(context).canPop() && widget.automaticallyImplyLeading;
    return SliverAppBar(
      pinned: true,
      automaticallyImplyLeading: false,
      title: ValueListenableBuilder<double>(
        builder: (context, collapsedRatio, child) {
          return Opacity(
            opacity: 1.0 - collapsedRatio.normalize(0.0, _titleCrossFadePoint),
            child: child,
          );
        },
        valueListenable: collapsedRatio,
        child: TitleText(
          widget.title,
          style: titleTextStyle,
        ),
      ),
      leading: widget.leading ?? (showBackButton ? const BackIconButton() : null),
      titleSpacing: widget.leading == null && !showBackButton ? null : 0.0,
      actions: widget.actions,
      titleTextStyle: titleTextStyle,
      collapsedHeight: toolbarHeight,
      expandedHeight: expandedHeight,
      flexibleSpace: LayoutBuilder(
        builder: (context, constraints) {
          /// Calculate the flexibleSpace's collapsed ratio, used to animate titles
          double minHeight = topPadding + toolbarHeight;
          final maxHeight = topPadding + expandedHeight;
          final scrollRange = maxHeight - minHeight;

          if (scrollRange > maxScrollExtent && maxScrollExtent > 0.0) {
            /// maxScrollExtent not sufficient to perform full animation, speeding up the process by
            /// providing a minHeight based on the available scroll range.
            minHeight = maxHeight - maxScrollExtent;
          }
          final collapsedRatio = constraints.maxHeight.normalize(minHeight, maxHeight).toDouble();

          /// Notify the [collapsedRatio] ValueNotifier so the SliverAppBar.title can be animated.
          WidgetsBinding.instance.addPostFrameCallback((duration) => this.collapsedRatio.value = collapsedRatio);

          /// Calculate the opacity of the expanded title
          final expandedTextOpacity = collapsedRatio.normalize(_titleCrossFadePoint, 1.0).toDouble();

          /// Render the flexible space, which includes the (optional) progress bar and expanded title
          return Stack(
            children: [
              _buildPositionedProgressBar(context),
              FlexibleSpaceBar(
                expandedTitleScale: 1,
                centerTitle: false,
                titlePadding: _expandedTitlePadding,
                title: Opacity(
                  opacity: expandedTextOpacity,
                  child: SafeArea(
                    top: false,
                    bottom: false,
                    child: TitleText(
                      widget.title,
                      style: titleTextStyle,
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

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _textHeightCache = null;
  }

  Widget _buildPositionedProgressBar(BuildContext context) {
    if (widget.progress == null) return const SizedBox.shrink();
    return Positioned(
      top: context.mediaQuery.padding.top + toolbarHeight - kStepIndicatorHeight,
      left: 0,
      right: 0,
      child: SafeArea(
        top: false,
        bottom: false,
        child: StepperIndicator(
          currentStep: widget.progress!.currentStep,
          totalSteps: widget.progress!.totalSteps,
        ),
      ),
    );
  }

  double _getTextHeight(String text) => _textHeightCache ?? _calculateTextHeight(text);

  double _calculateTextHeight(String text) {
    final TextPainter tp = TextPainter(
      text: TextSpan(text: text, style: context.theme.appBarTheme.titleTextStyle),
      textDirection: TextDirection.ltr,
      textScaler: MediaQuery.of(context).textScaler,
    );
    tp.layout(maxWidth: MediaQuery.of(context).size.width - _expandedTitlePadding.horizontal);
    return _textHeightCache = tp.height;
  }
}
