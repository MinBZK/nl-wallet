import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../data/repository/card/data_attribute_repository.dart';
import '../data/repository/card/impl/data_attribute_repository_impl.dart';
import '../data/repository/card/impl/timeline_attribute_repository_impl.dart';
import '../data/repository/card/impl/wallet_card_repository_impl.dart';
import '../data/repository/card/timeline_attribute_repository.dart';
import '../data/repository/card/wallet_card_repository.dart';
import '../data/repository/configuration/configuration_repository.dart';
import '../data/repository/configuration/core/core_configuration_repository.dart';
import '../data/repository/configuration/mock/mock_configuration_repository.dart';
import '../data/repository/disclosure/core/core_disclosure_request_repository.dart';
import '../data/repository/disclosure/disclosure_request_repository.dart';
import '../data/repository/disclosure/mock/mock_disclosure_request_repository.dart';
import '../data/repository/issuance/core/core_issuance_response_repository.dart';
import '../data/repository/issuance/issuance_response_repository.dart';
import '../data/repository/issuance/mock/mock_issuance_response_repository.dart';
import '../data/repository/language/impl/language_repository_impl.dart';
import '../data/repository/language/language_repository.dart';
import '../data/repository/organization/impl/organization_repository_impl.dart';
import '../data/repository/organization/organization_repository.dart';
import '../data/repository/pid/core/core_pid_repository.dart';
import '../data/repository/pid/mock/mock_pid_repository.dart';
import '../data/repository/pid/pid_repository.dart';
import '../data/repository/qr/core/core_qr_repository.dart';
import '../data/repository/qr/mock/mock_qr_repository.dart';
import '../data/repository/qr/qr_repository.dart';
import '../data/repository/sign/core/core_sign_request_repository.dart';
import '../data/repository/sign/mock/mock_sign_request_repository.dart';
import '../data/repository/sign/sign_request_repository.dart';
import '../data/repository/wallet/core/core_wallet_repository.dart';
import '../data/repository/wallet/mock/mock_wallet_repository.dart';
import '../data/repository/wallet/wallet_repository.dart';

/// This widget is responsible for initializing and providing all `repositories`.
/// Most likely to be used once at the top (app) level.
class WalletRepositoryProvider extends StatelessWidget {
  final Widget child;
  final bool provideMocks;

  const WalletRepositoryProvider({required this.child, this.provideMocks = false, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<WalletRepository>(
          create: (context) => provideMocks
              ? MockWalletRepository(context.read())
              : CoreWalletRepository(context.read(), context.read()),
        ),
        RepositoryProvider<OrganizationRepository>(
          create: (context) => OrganizationRepositoryImpl(context.read()),
        ),
        RepositoryProvider<WalletCardRepository>(create: (context) => WalletCardRepositoryImpl(context.read())),
        RepositoryProvider<DataAttributeRepository>(
          create: (context) => DataAttributeRepositoryImpl(context.read()),
        ),
        RepositoryProvider<TimelineAttributeRepository>(
          create: (context) => TimelineAttributeRepositoryImpl(context.read()),
        ),
        RepositoryProvider<DisclosureRequestRepository>(
          create: (context) =>
              provideMocks ? MockDisclosureRequestRepository(context.read()) : CoreDisclosureRequestRepository(),
        ),
        RepositoryProvider<ConfigurationRepository>(
          create: (context) =>
              provideMocks ? MockConfigurationRepository() : CoreConfigurationRepository(context.read()),
        ),
        RepositoryProvider<QrRepository>(
          create: (context) => provideMocks ? MockQrRepository() : CoreQrRepository(),
        ),
        RepositoryProvider<IssuanceResponseRepository>(
          create: (context) =>
              provideMocks ? MockIssuanceResponseRepository(context.read()) : CoreIssuanceResponseRepository(),
        ),
        RepositoryProvider<SignRequestRepository>(
          create: (context) => provideMocks ? MockSignRequestRepository(context.read()) : CoreSignRequestRepository(),
        ),
        RepositoryProvider<LanguageRepository>(
          create: (context) => LanguageRepositoryImpl(context.read(), AppLocalizations.supportedLocales),
        ),
        RepositoryProvider<PidRepository>(
          create: (context) => provideMocks
              ? MockPidRepository()
              : CorePidRepository(context.read(), context.read(), context.read(), context.read()),
        ),
      ],
      child: child,
    );
  }
}
