import 'package:flutter/material.dart';

import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../mapper.dart';

class InteractionErrorStatusIconMapper extends Mapper<InteractionStatus, IconData?> {
  @override
  IconData? map(InteractionStatus input) {
    switch (input) {
      case InteractionStatus.rejected:
        return Icons.block_outlined;
      case InteractionStatus.failed:
        return Icons.error_outline;
      default:
        return null;
    }
  }
}
