import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/app_image_data.dart';
import '../../../../domain/model/attribute/attribute.dart';
import '../../mapper.dart';

class CardAttributeValueMapper extends Mapper<core.AttributeValue, AttributeValue> {
  final Mapper<core.Image, AppImageData> _imageMapper;

  CardAttributeValueMapper(this._imageMapper);

  @override
  AttributeValue map(core.AttributeValue input) {
    return switch (input) {
      core.AttributeValue_String(:final value) => StringValue(value),
      core.AttributeValue_Boolean(:final value) => BooleanValue(value),
      core.AttributeValue_Number(:final value) => NumberValue(_simplify(value)),
      core.AttributeValue_Array(:final value) => ArrayValue(value.map(map).toList()),
      core.AttributeValue_Null() => NullValue(),
      core.AttributeValue_Date(:final value) => DateValue(DateTime.parse(value)),
      core.AttributeValue_Bytes(:final value) => BytesValue(value),
      core.AttributeValue_Image(:final value) => ImageValue(_imageMapper.map(value)),
      core.AttributeValue_Map(:final value) => MapValue({for (final (key, value) in value) key: map(value)}),
    };
  }

  /// The core provides every number as a [double], whole numbers included. Narrow those back to an
  /// [int] so that e.g. a mileage renders as '123' instead of '123.0'.
  num _simplify(double value) => value.isFinite && value % 1 == 0 ? value.toInt() : value;
}
