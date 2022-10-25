import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';

class InteractionDetector extends SingleChildRenderObjectWidget {
  final VoidCallback onInteraction;

  const InteractionDetector({required this.onInteraction, super.child, super.key});

  @override
  RenderObject createRenderObject(BuildContext context) => InteractionRenderBox(onInteraction: onInteraction);
}

class InteractionRenderBox extends RenderProxyBox {
  final VoidCallback onInteraction;

  InteractionRenderBox({required this.onInteraction, RenderBox? child}) : super(child);

  @override
  bool hitTest(BoxHitTestResult result, {required Offset position}) {
    onInteraction();
    return super.hitTest(result, position: position);
  }
}
