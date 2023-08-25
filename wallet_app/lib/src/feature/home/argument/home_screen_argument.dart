import 'package:json_annotation/json_annotation.dart';

import '../../../domain/model/wallet_card.dart';

part 'home_screen_argument.g.dart';

@JsonSerializable(explicitToJson: true)
class HomeScreenArgument {
  final List<WalletCard> cards;

  const HomeScreenArgument({required this.cards});

  factory HomeScreenArgument.fromJson(Map<String, dynamic> json) => _$HomeScreenArgumentFromJson(json);

  Map<String, dynamic> toJson() => _$HomeScreenArgumentToJson(this);
}
