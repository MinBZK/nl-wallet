import 'dart:ui';

import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:rxdart/rxdart.dart';

/// A service that listens for and propagates semantics actions.
///
/// This service intercepts [SemanticsActionEvent]s from the platform,
/// decodes their arguments if necessary, and then re-dispatches them
/// through the [WidgetsBinding] instance. It also provides a stream
/// of these events for other parts of the application to consume.
class SemanticsEventService {
  final PublishSubject<SemanticsActionEvent> _events = PublishSubject();

  /// A stream of [SemanticsActionEvent]s that are dispatched by the system.
  ///
  /// Consumers can listen to this stream to react to semantics actions
  /// as they occur.
  Stream<SemanticsActionEvent> get actionEventStream => _events;

  SemanticsEventService() {
    PlatformDispatcher.instance.onSemanticsActionEvent = (SemanticsActionEvent action) {
      try {
        _notify(action);
        final Object? arguments = action.arguments;
        // Decode the [SemanticsActionEvent] before passing it on. Needed to avoid ex. & support scroll like events.
        final SemanticsActionEvent decodedAction = arguments is ByteData
            ? action.copyWith(arguments: const StandardMessageCodec().decodeMessage(arguments))
            : action;
        WidgetsBinding.instance.performSemanticsAction(decodedAction);
      } catch (ex) {
        Fimber.e('Failed to propagate semantics action: $action', ex: ex);
      }
    };
  }

  void _notify(SemanticsActionEvent event) => _events.add(event);
}
