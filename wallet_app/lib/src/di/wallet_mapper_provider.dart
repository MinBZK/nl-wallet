import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../domain/model/attribute/data_attribute.dart';
import '../domain/model/pin/pin_validation_error.dart';
import '../domain/model/wallet_card.dart';
import '../util/mapper/card/attribute/card_attribute_label_mapper.dart';
import '../util/mapper/card/attribute/card_attribute_mapper.dart';
import '../util/mapper/card/attribute/card_attribute_value_mapper.dart';
import '../util/mapper/card/card_mapper.dart';
import '../util/mapper/card/card_subtitle_mapper.dart';
import '../util/mapper/locale_mapper.dart';
import '../util/mapper/mapper.dart';
import '../util/mapper/pid/core_pid_attribute_mapper.dart';
import '../util/mapper/pid/mock_pid_attribute_mapper.dart';
import '../util/mapper/pid/pid_attribute_mapper.dart';
import '../util/mapper/pin/pin_validation_error_mapper.dart';
import '../wallet_core/error/core_error.dart';
import '../wallet_core/error/core_error_mapper.dart';
import '../wallet_core/wallet_core.dart';

class WalletMapperProvider extends StatelessWidget {
  final Widget child;
  final bool provideMocks;

  const WalletMapperProvider({required this.child, this.provideMocks = false, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        /// Core mappers
        RepositoryProvider<Mapper<String, CoreError>>(
          create: (context) => CoreErrorMapper(),
        ),

        /// Card mappers
        RepositoryProvider<LocaleMapper<Card, WalletCard>>(
          create: (context) => CardMapper(context.read(), context.read()),
        ),
        RepositoryProvider<LocaleMapper<Card, String>>(
          create: (context) => CardSubtitleMapper(context.read()),
        ),

        /// Card attribute mappers
        RepositoryProvider<LocaleMapper<List<LocalizedString>, String>>(
          create: (context) => CardAttributeLabelMapper(),
        ),
        RepositoryProvider<LocaleMapper<CardAttribute, DataAttribute>>(
          create: (context) => CardAttributeMapper(context.read(), context.read()),
        ),
        RepositoryProvider<LocaleMapper<CardValue, String>>(
          create: (context) => CardAttributeValueMapper(),
        ),

        /// Pid mappers
        RepositoryProvider<PidAttributeMapper>(
          create: (context) => (provideMocks ? MockPidAttributeMapper() : CorePidAttributeMapper()),
        ),

        /// Pin mappers
        RepositoryProvider<Mapper<PinValidationResult, PinValidationError?>>(
          create: (context) => PinValidationErrorMapper(),
        ),
      ],
      child: child,
    );
  }
}
