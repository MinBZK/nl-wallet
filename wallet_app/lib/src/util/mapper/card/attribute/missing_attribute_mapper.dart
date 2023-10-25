import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/attribute/requested_attribute.dart';
import '../../../../wallet_core/wallet_core.dart';
import '../../locale_mapper.dart';

class MissingAttributeMapper extends LocaleMapper<MissingAttribute, RequestedAttribute> {
  final LocaleMapper<List<LocalizedString>, String> _attributeLabelMapper;

  MissingAttributeMapper(this._attributeLabelMapper);

  @override
  RequestedAttribute map(Locale locale, MissingAttribute input) {
    return RequestedAttribute(
      key: '', // FIXME: Remove notion of key in requested attribute?
      label: _attributeLabelMapper.map(locale, input.labels),
      valueType: AttributeValueType.text,
    );
  }
}
