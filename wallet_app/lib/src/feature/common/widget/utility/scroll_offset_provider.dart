import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:provider/provider.dart';

import '../../../../util/extension/build_context_extension.dart';

/// Widget that provides a [ScrollOffset] to it's descendants. By default the [ScrollOffset] is
/// updated based on any incoming [ScrollNotification]s. This behaviour can be overridden with the
/// [observeScrollNotifications] flag.
class ScrollOffsetProvider extends StatefulWidget {
  final Widget child;
  final String debugLabel;
  final bool observeScrollNotifications;

  const ScrollOffsetProvider({
    required this.child,
    this.debugLabel = '',
    this.observeScrollNotifications = true,
    super.key,
  });

  @override
  State<ScrollOffsetProvider> createState() => _ScrollOffsetProviderState();
}

class _ScrollOffsetProviderState extends State<ScrollOffsetProvider> {
  Orientation? _orientation;

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
      create: (context) => ScrollOffset(widget.debugLabel),
      builder: (context, child) {
        if (child == null) return const SizedBox.shrink();
        if (_orientation != context.mediaQuery.orientation) {
          _orientation = context.mediaQuery.orientation;
          SchedulerBinding.instance.addPostFrameCallback((_) => context.read<ScrollOffset>().reset());
        }
        if (!widget.observeScrollNotifications) return child;
        return NotificationListener<ScrollNotification>(
          onNotification: (notification) {
            final scrollOffset = context.read<ScrollOffset>();
            scrollOffset.offset = notification.metrics.hasPixels ? notification.metrics.pixels : 0;
            scrollOffset.maxScrollExtent = notification.metrics.hasContentDimensions
                ? notification.metrics.maxScrollExtent
                : 0;
            return false;
          },
          child: child,
        );
      },
      child: widget.child,
    );
  }
}

/// A simple object to provide a [ScrollController]s offset to other interested widgets that
/// could not otherwise observe it. E.g. useful for sibling widgets that can't rely on the
/// [PrimaryScrollController] or [ScrollNotification]s. In our case it is relevant in the
/// disclose/issue/sign flows, where there is no clear primary [ScrollController] and the
/// [WalletAppBar] can't observe the [ScrollNotification]s because it's a sibling, and not
/// a parent of the scrolling content.
class ScrollOffset extends ChangeNotifier {
  final String debugLabel;

  double _offset = 0;
  double _maxScrollExtent = 0;

  ScrollOffset(this.debugLabel);

  double get offset => _offset;

  double get maxScrollExtent => _maxScrollExtent;

  set offset(double value) {
    if (_offset == value) return;
    _offset = value;
    notifyListeners();
  }

  set maxScrollExtent(double value) {
    if (_maxScrollExtent == value) return;
    _maxScrollExtent = value;
    notifyListeners();
  }

  @override
  String toString() => 'ScrollOffset for $debugLabel. Offset: $_offset, MaxScrollExtent: $maxScrollExtent';

  /// Resets the [ScrollOffset] to it's initial values (i.e. 0)
  /// Can be useful when scrolling happens within a [PageView], as
  /// navigating to the next page might not trigger an automatic update.
  void reset() {
    bool changed = false;
    if (_offset != 0) {
      _offset = 0;
      changed = true;
    }
    if (_maxScrollExtent != 0) {
      _maxScrollExtent = 0;
      changed = true;
    }
    if (changed) notifyListeners();
  }
}
