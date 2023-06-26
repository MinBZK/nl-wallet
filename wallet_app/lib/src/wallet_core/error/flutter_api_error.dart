import 'package:json_annotation/json_annotation.dart';

part 'flutter_api_error.g.dart';

@JsonSerializable()
class FlutterApiError {
  FlutterApiErrorType type;
  String? description;

  FlutterApiError({required this.type, this.description});

  factory FlutterApiError.fromJson(Map<String, dynamic> json) => _$FlutterApiErrorFromJson(json);

  Map<String, dynamic> toJson() => _$FlutterApiErrorToJson(this);

  @override
  String toString() => 'FlutterApiError{type: ${type.name}, description: $description}';
}

enum FlutterApiErrorType {
  @JsonValue('Generic')
  generic,
  @JsonValue('Networking')
  networking
}
