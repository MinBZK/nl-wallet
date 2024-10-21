import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:wallet_core/core.dart' as core;
import 'package:wallet_core/core.dart' show Card, CardValue, DisclosureCard, LocalizedString, PinValidationResult;
import 'package:wallet_mock/mock.dart' as core show Document;

import '../domain/model/app_image_data.dart';
import '../domain/model/attribute/attribute.dart';
import '../domain/model/attribute/data_attribute.dart';
import '../domain/model/attribute/missing_attribute.dart';
import '../domain/model/card_config.dart';
import '../domain/model/card_front.dart';
import '../domain/model/disclosure/disclosure_session_type.dart';
import '../domain/model/document.dart';
import '../domain/model/event/wallet_event.dart';
import '../domain/model/organization.dart';
import '../domain/model/pin/pin_validation_error.dart';
import '../domain/model/policy/organization_policy.dart';
import '../domain/model/policy/policy.dart';
import '../domain/model/wallet_card.dart';
import '../util/mapper/card/attribute/card_attribute_mapper.dart';
import '../util/mapper/card/attribute/card_attribute_value_mapper.dart';
import '../util/mapper/card/attribute/localized_labels_mapper.dart';
import '../util/mapper/card/attribute/missing_attribute_mapper.dart';
import '../util/mapper/card/card_config_mapper.dart';
import '../util/mapper/card/card_front_mapper.dart';
import '../util/mapper/card/card_mapper.dart';
import '../util/mapper/card/card_subtitle_mapper.dart';
import '../util/mapper/card/disclosure_card_mapper.dart';
import '../util/mapper/context_mapper.dart';
import '../util/mapper/disclosure/disclosure_session_type_mapper.dart';
import '../util/mapper/disclosure/disclosure_type_mapper.dart';
import '../util/mapper/document/document_mapper.dart';
import '../util/mapper/event/wallet_event_mapper.dart';
import '../util/mapper/image/image_mapper.dart';
import '../util/mapper/mapper.dart';
import '../util/mapper/organization/organization_mapper.dart';
import '../util/mapper/pid/core_pid_attribute_mapper.dart';
import '../util/mapper/pid/mock_pid_attribute_mapper.dart';
import '../util/mapper/pid/pid_attribute_mapper.dart';
import '../util/mapper/pin/pin_validation_error_mapper.dart';
import '../util/mapper/policy/policy_body_text_mapper.dart';
import '../util/mapper/policy/request_policy_mapper.dart';
import '../wallet_core/error/core_error.dart';
import '../wallet_core/error/core_error_mapper.dart';

class WalletMapperProvider extends StatelessWidget {
  final Widget child;
  final bool provideMocks;

  const WalletMapperProvider({required this.child, this.provideMocks = false, super.key});

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

        /// Organization / Relying party mappers
        RepositoryProvider<Mapper<core.Organization, Organization>>(
          create: (context) => OrganizationMapper(context.read(), context.read()),
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
          create: (context) => CardMapper(context.read(), context.read(), context.read(), context.read()),
        ),
        RepositoryProvider<Mapper<DisclosureCard, WalletCard>>(
          create: (context) => DisclosureCardMapper(context.read()),
        ),

        /// Policy
        RepositoryProvider<Mapper<core.RequestPolicy, Policy>>(
          create: (context) => RequestPolicyMapper(),
        ),
        RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
          create: (context) => PolicyBodyTextMapper(),
        ),

        /// Document
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

        /// Disclosure mappers
        RepositoryProvider<Mapper<core.DisclosureSessionType, DisclosureSessionType>>(
          create: (context) => DisclosureSessionTypeMapper(),
        ),
        RepositoryProvider<Mapper<core.DisclosureType, DisclosureType>>(
          create: (context) => DisclosureTypeMapper(),
        ),

        /// Event mapper
        RepositoryProvider<Mapper<core.WalletEvent, WalletEvent>>(
          create: (context) => WalletEventMapper(
            context.read(),
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
