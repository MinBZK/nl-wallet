import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/attribute/attribute.dart';
import '../../mapper.dart';

class MissingAttributeMapper extends Mapper<core.MissingAttribute, MissingAttribute> {
  final Mapper<List<core.LocalizedString>, LocalizedText> _localizedLabelsMapper;

  MissingAttributeMapper(this._localizedLabelsMapper);

  @override
  MissingAttribute map(core.MissingAttribute input) =>
      MissingAttribute(label: _localizedLabelsMapper.map(input.labels));
}
