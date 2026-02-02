import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../domain/model/card/wallet_card.dart';

part 'dashboard_screen_argument.freezed.dart';
part 'dashboard_screen_argument.g.dart';

@Freezed(copyWith: false)
abstract class DashboardScreenArgument with _$DashboardScreenArgument {
  const factory DashboardScreenArgument({
    required List<WalletCard> cards,
  }) = _DashboardScreenArgument;

  factory DashboardScreenArgument.fromJson(Map<String, dynamic> json) => _$DashboardScreenArgumentFromJson(json);
}
