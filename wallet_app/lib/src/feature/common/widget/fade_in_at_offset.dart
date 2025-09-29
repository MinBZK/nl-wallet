import 'dart:async';
import 'dart:math';

import 'package:after_layout/after_layout.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../util/extension/num_extensions.dart';
import 'utility/scroll_offset_provider.dart';

/// Widget that fades in (using opacity) the provided [child] based on the scroll offset that
/// it was able to resolve. The scroll offset used for the animation is resolved in based on the
/// following priority:
///   1. Any [ScrollOffset] provided by an ancestor. E.g. using the provided [ScrollOffsetProvider]
///   2. The scroll offset of the provided [scrollController]
///   3. The scroll offset of the [PrimaryScrollController]
/// If none of the above can be resolved, a [UnsupportedError] is thrown.
class FadeInAtOffset extends StatefulWidget {
  /// The offset at which the [child] should start to appear
  final double appearOffset;

  /// The offset at which the [child] should be fully visible
  final double visibleOffset;

  /// The widget that should be fully visible (opacity) at [visibleOffset]
  final Widget child;

  /// The scrollController to observe, if none is provided the widget looks for the PrimaryScrollController.
  final ScrollController? scrollController;

  const FadeInAtOffset({
    this.appearOffset = 0,
    required this.visibleOffset,
    required this.child,
    this.scrollController,
    super.key,
  }) : assert(
         appearOffset < visibleOffset,
         'appear offset should be smaller than the offset at which the child is fully visible',
       );

  @override
  State<FadeInAtOffset> createState() => _FadeInAtOffsetState();
}

class _FadeInAtOffsetState extends State<FadeInAtOffset> with AfterLayoutMixin<FadeInAtOffset> {
  bool _afterFirstLayout = false;
  ScrollController? _scrollController;

  bool get scrollControllerHasClients => _scrollController?.hasClients ?? false;

  double get offset =>
      context.read<ScrollOffset?>()?.offset ?? (scrollControllerHasClients ? _scrollController!.offset : 0);

  double get maxScrollExtent =>
      context.read<ScrollOffset?>()?.maxScrollExtent ??
      (scrollControllerHasClients ? _scrollController!.position.maxScrollExtent : double.infinity);

  @override
  Widget build(BuildContext context) {
    final scrollOffset = context.watch<ScrollOffset?>();

    /// Check if we are ready to build, as before the first layout the _scrollController will not be initialized.
    if (scrollOffset == null && !_afterFirstLayout) return const SizedBox.shrink();

    double startAppearingAt = widget.appearOffset;
    double completelyVisibleAt = widget.visibleOffset;

    // If don't have enough room to scroll to reach the 'start animation' breakpoint we simply
    // never show the [child], as this should mean the original title always stays visible.
    if (maxScrollExtent <= startAppearingAt) return const SizedBox.shrink();

    if (maxScrollExtent > 0 /* if maxScrollExtent is 0, we only animate for the overscroll */ ) {
      // We make sure the widget will always animate to 100% opacity by comparing it with the maximum scrollable extend.
      startAppearingAt = min(widget.appearOffset, maxScrollExtent - 1);
      completelyVisibleAt = min(widget.visibleOffset, maxScrollExtent);
    }

    // Exclude the widget from focus and pointer events when it's not visible.
    final completelyInvisible = offset <= startAppearingAt;
    return Offstage(
      offstage: completelyInvisible,
      child: Opacity(
        opacity: offset.normalize(startAppearingAt, completelyVisibleAt).toDouble(),
        child: widget.child,
      ),
    );
  }

  @override
  void didUpdateWidget(covariant FadeInAtOffset oldWidget) {
    super.didUpdateWidget(oldWidget);

    // If a ScrollOffset is provided, we don't manage our own listeners.
    if (context.read<ScrollOffset?>() != null) {
      // Make sure to clean up if we were previously managing one.
      if (_scrollController != null) {
        _scrollController!.removeListener(_onScroll);
        _scrollController = null;
      }
      return;
    }

    // Re-attach to scroll controller if anything changed
    final scrollController = widget.scrollController ?? PrimaryScrollController.of(context);
    if (scrollController != _scrollController || widget.scrollController != oldWidget.scrollController) {
      _scrollController?.removeListener(_onScroll);
      _scrollController = scrollController;
      _scrollController?.addListener(_onScroll);
    }
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();

    // If we are not monitoring a [ScrollController] this is already handled by [ScrollOffsetProvider]
    if (_scrollController != null) {
      MediaQuery.of(context).orientation; // Important to monitor the orientation changes.
      WidgetsBinding.instance.addPostFrameCallback((_) {
        // This helps 'sync' the scrollOffset after an orientation change
        if (_afterFirstLayout && context.mounted) setState(() {});
      });
    }
  }

  @override
  FutureOr<void> afterFirstLayout(BuildContext context) {
    if (context.read<ScrollOffset?>() == null) {
      /// No ancestor providing [ScrollOffset], resolve scroll from the scrollController
      _scrollController = widget.scrollController ?? PrimaryScrollController.of(context);
      _scrollController?.addListener(_onScroll);
    }
    _afterFirstLayout = true;
  }

  @override
  void dispose() {
    _scrollController?.removeListener(_onScroll);
    super.dispose();
  }

  void _onScroll() => setState(() {});
}
