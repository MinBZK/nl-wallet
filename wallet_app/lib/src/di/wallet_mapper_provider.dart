import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:wallet_core/core.dart' as core;
import 'package:wallet_core/core.dart'
    show Card, CardValue, LocalizedString, PinValidationResult, DisclosureCard, WalletEvent;
import 'package:wallet_mock/mock.dart' as core show Document;

import '../data/repository/organization/organization_repository.dart';
import '../domain/model/app_image_data.dart';
import '../domain/model/attribute/attribute.dart';
import '../domain/model/attribute/data_attribute.dart';
import '../domain/model/attribute/missing_attribute.dart';
import '../domain/model/card_config.dart';
import '../domain/model/card_front.dart';
import '../domain/model/document.dart';
import '../domain/model/pin/pin_validation_error.dart';
import '../domain/model/policy/policy.dart';
import '../domain/model/timeline/timeline_attribute.dart';
import '../domain/model/wallet_card.dart';
import '../util/mapper/card/attribute/card_attribute_mapper.dart';
import '../util/mapper/card/attribute/card_attribute_value_mapper.dart';
import '../util/mapper/card/attribute/localized_labels_mapper.dart';
import '../util/mapper/card/attribute/missing_attribute_mapper.dart';
import '../util/mapper/card/card_config_mapper.dart';
import '../util/mapper/card/card_front_mapper.dart';
import '../util/mapper/card/card_mapper.dart';
import '../util/mapper/card/card_subtitle_mapper.dart';
import '../util/mapper/card/requested_card_mapper.dart';
import '../util/mapper/document/document_mapper.dart';
import '../util/mapper/history/wallet_event_mapper.dart';
import '../util/mapper/image/image_mapper.dart';
import '../util/mapper/mapper.dart';
import '../util/mapper/organization/organization_mapper.dart';
import '../util/mapper/pid/core_pid_attribute_mapper.dart';
import '../util/mapper/pid/mock_pid_attribute_mapper.dart';
import '../util/mapper/pid/pid_attribute_mapper.dart';
import '../util/mapper/pin/pin_validation_error_mapper.dart';
import '../util/mapper/policy/request_policy.dart';
import '../wallet_core/error/core_error.dart';
import '../wallet_core/error/core_error_mapper.dart';

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
        RepositoryProvider<Mapper<core.Image, AppImageData>>(
          create: (context) => ImageMapper(),
        ),

        /// Card attribute mappers
        RepositoryProvider<Mapper<List<LocalizedString>, LocalizedText>>(
          create: (context) => LocalizedLabelsMapper(),
        ),
        RepositoryProvider<Mapper<CardValue, AttributeValue>>(
          create: (context) => CardAttributeValueMapper(),
        ),
        RepositoryProvider<Mapper<CardAttributeWithDocType, DataAttribute>>(
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
        RepositoryProvider<Mapper<String, CardConfig>>(
          create: (context) => CardConfigMapper(),
        ),
        RepositoryProvider<Mapper<Card, WalletCard>>(
          create: (context) => CardMapper(context.read(), context.read(), context.read()),
        ),
        RepositoryProvider<Mapper<DisclosureCard, WalletCard>>(
          create: (context) => DisclosureCardMapper(context.read()),
        ),

        /// Organization / Relying party mappers
        RepositoryProvider<Mapper<core.Organization, Organization>>(
          create: (context) => OrganizationMapper(context.read(), context.read()),
        ),

        /// Policy
        RepositoryProvider<Mapper<core.RequestPolicy, Policy>>(
          create: (context) => RequestPolicyMapper(),
        ),

        /// Policy
        RepositoryProvider<Mapper<core.Document, Document>>(
          create: (context) => DocumentMapper(),
        ),

        /// Pid mappers
        RepositoryProvider<PidAttributeMapper>(
          create: (context) => (provideMocks ? MockPidAttributeMapper() : CorePidAttributeMapper()),
        ),

        /// Pin mappers
        RepositoryProvider<Mapper<PinValidationResult, PinValidationError?>>(
          create: (context) => PinValidationErrorMapper(),
        ),

        /// Transaction mapper
        RepositoryProvider<Mapper<WalletEvent, TimelineAttribute>>(
          create: (context) => WalletEventMapper(
            context.read(),
            context.read(),
            context.read(),
            context.read(),
            context.read(),
          ),
        ),
      ],
      child: child,
    );
  }
}
