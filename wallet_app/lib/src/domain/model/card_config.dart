import 'package:equatable/equatable.dart';
import 'package:json_annotation/json_annotation.dart';

part 'card_config.g.dart';

@JsonSerializable()
class CardConfig extends Equatable {
  final bool updatable;
  final bool removable;

  const CardConfig({
    this.updatable = false,
    this.removable = false,
  });

  factory CardConfig.fromJson(Map<String, dynamic> json) => _$CardConfigFromJson(json);

  Map<String, dynamic> toJson() => _$CardConfigToJson(this);

  @override
  List<Object?> get props => [updatable, removable];
}
