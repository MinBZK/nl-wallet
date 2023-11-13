import '../../../../domain/model/attribute/missing_attribute.dart';
import '../../../../domain/model/localized_text.dart';
import '../../../../wallet_core/wallet_core.dart' as core;
import '../../mapper.dart';

class MissingAttributeMapper extends Mapper<core.MissingAttribute, MissingAttribute> {
  final Mapper<List<core.LocalizedString>, LocalizedText> _localizedLabelsMapper;

  MissingAttributeMapper(this._localizedLabelsMapper);

  @override
  MissingAttribute map(core.MissingAttribute input) =>
      MissingAttribute(label: _localizedLabelsMapper.map(input.labels));
}
