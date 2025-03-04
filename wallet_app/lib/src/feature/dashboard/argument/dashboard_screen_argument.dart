import 'package:equatable/equatable.dart';
import 'package:json_annotation/json_annotation.dart';

import '../../../domain/model/card/wallet_card.dart';

part 'dashboard_screen_argument.g.dart';

@JsonSerializable(explicitToJson: true)
class DashboardScreenArgument extends Equatable {
  final List<WalletCard> cards;

  const DashboardScreenArgument({required this.cards});

  factory DashboardScreenArgument.fromJson(Map<String, dynamic> json) => _$DashboardScreenArgumentFromJson(json);

  Map<String, dynamic> toJson() => _$DashboardScreenArgumentToJson(this);

  @override
  List<Object?> get props => [cards];
}
