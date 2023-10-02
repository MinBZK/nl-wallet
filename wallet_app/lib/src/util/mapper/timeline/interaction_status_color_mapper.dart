import 'package:flutter/material.dart';

import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../extension/build_context_extension.dart';
import '../context_mapper.dart';

class InteractionStatusColorMapper extends ContextMapper<InteractionStatus, Color> {
  @override
  Color map(BuildContext context, InteractionStatus input) {
    switch (input) {
      case InteractionStatus.success:
        return context.colorScheme.onBackground;
      case InteractionStatus.failed:
        return context.colorScheme.error;
      case InteractionStatus.rejected:
        return context.colorScheme.error;
    }
  }
}
