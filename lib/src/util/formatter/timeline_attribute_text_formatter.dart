import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/model/timeline_attribute.dart';
import '../mapper/timeline_attribute_type_text_mapper.dart';

class TimelineAttributeTextFormatter {
  static String format(AppLocalizations locale, TimelineAttribute attribute) {
    final String typeText = TimelineAttributeTypeTextMapper.map(locale, attribute);
    if (attribute is InteractionAttribute) return locale.cardTimelineInteraction(typeText, attribute.organization);
    if (attribute is OperationAttribute) return locale.cardTimelineOperation(typeText, attribute.description);
    throw ('Unsupported attribute: $attribute');
  }
}
