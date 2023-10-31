import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../data/repository/organization/organization_repository.dart';
import '../domain/model/attribute/attribute.dart';
import '../domain/model/attribute/data_attribute.dart';
import '../domain/model/attribute/missing_attribute.dart';
import '../domain/model/card_front.dart';
import '../domain/model/pin/pin_validation_error.dart';
import '../domain/model/wallet_card.dart';
import '../util/mapper/card/attribute/card_attribute_mapper.dart';
import '../util/mapper/card/attribute/card_attribute_value_mapper.dart';
import '../util/mapper/card/attribute/localized_labels_mapper.dart';
import '../util/mapper/card/attribute/missing_attribute_mapper.dart';
import '../util/mapper/card/card_front_mapper.dart';
import '../util/mapper/card/card_mapper.dart';
import '../util/mapper/card/card_subtitle_mapper.dart';
import '../util/mapper/card/requested_card_mapper.dart';
import '../util/mapper/mapper.dart';
import '../util/mapper/organization/relying_party_mapper.dart';
import '../util/mapper/pid/core_pid_attribute_mapper.dart';
import '../util/mapper/pid/mock_pid_attribute_mapper.dart';
import '../util/mapper/pid/pid_attribute_mapper.dart';
import '../util/mapper/pin/pin_validation_error_mapper.dart';
import '../wallet_core/error/core_error.dart';
import '../wallet_core/error/core_error_mapper.dart';
import '../wallet_core/wallet_core.dart' as core;
import '../wallet_core/wallet_core.dart'
    show Card, CardAttribute, CardValue, LocalizedString, PinValidationResult, RelyingParty, RequestedCard;

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

        /// Card attribute mappers
        RepositoryProvider<Mapper<List<LocalizedString>, LocalizedText>>(
          create: (context) => LocalizedLabelsMapper(),
        ),
        RepositoryProvider<Mapper<CardValue, AttributeValue>>(
          create: (context) => CardAttributeValueMapper(),
        ),
        RepositoryProvider<Mapper<CardAttribute, DataAttribute>>(
          create: (context) => CardAttributeMapper(context.read(), context.read()),
        ),
        RepositoryProvider<Mapper<core.MissingAttribute, MissingAttribute>>(
          create: (context) => MissingAttributeMapper(context.read()),
        ),

        /// Card mappers
        RepositoryProvider<Mapper<Card, LocalizedText?>>(
          create: (context) => CardSubtitleMapper(context.read()),
        ),
        RepositoryProvider<Mapper<Card, CardFront>>(
          create: (context) => CardFrontMapper(context.read()),
        ),
        RepositoryProvider<Mapper<Card, WalletCard>>(
          create: (context) => CardMapper(context.read(), context.read()),
        ),
        RepositoryProvider<Mapper<RequestedCard, WalletCard>>(
          create: (context) => RequestedCardMapper(context.read(), context.read()),
        ),

        /// Organization / Relying party mappers
        RepositoryProvider<Mapper<RelyingParty, Organization>>(
          create: (context) => RelyingPartyMapper(),
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
