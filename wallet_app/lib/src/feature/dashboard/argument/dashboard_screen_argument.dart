import 'package:json_annotation/json_annotation.dart';

import '../../../domain/model/wallet_card.dart';

part 'dashboard_screen_argument.g.dart';

@JsonSerializable(explicitToJson: true)
class DashboardScreenArgument {
  final List<WalletCard> cards;

  const DashboardScreenArgument({required this.cards});

  factory DashboardScreenArgument.fromJson(Map<String, dynamic> json) => _$DashboardScreenArgumentFromJson(json);

  Map<String, dynamic> toJson() => _$DashboardScreenArgumentToJson(this);
}
