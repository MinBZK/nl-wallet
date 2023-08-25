import 'package:json_annotation/json_annotation.dart';
import 'package:equatable/equatable.dart';

part 'card_config.g.dart';

@JsonSerializable()
class CardConfig extends Equatable {
  final bool removable;

  const CardConfig({
    this.removable = true,
  });

  factory CardConfig.fromJson(Map<String, dynamic> json) => _$CardConfigFromJson(json);

  Map<String, dynamic> toJson() => _$CardConfigToJson(this);

  @override
  List<Object?> get props => [removable];
}
