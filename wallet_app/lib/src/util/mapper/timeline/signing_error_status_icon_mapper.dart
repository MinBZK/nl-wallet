import 'package:flutter/material.dart';

import '../../../domain/model/timeline/signing_timeline_attribute.dart';
import '../mapper.dart';

class SigningErrorStatusIconMapper extends Mapper<SigningStatus, IconData?> {
  @override
  IconData? map(SigningStatus input) {
    switch (input) {
      case SigningStatus.rejected:
        return Icons.block_outlined;
      default:
        return null;
    }
  }
}
