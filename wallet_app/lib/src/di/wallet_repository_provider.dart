import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:wallet_mock/mock.dart';

import '../data/repository/biometric/biometric_repository.dart';
import '../data/repository/biometric/core/core_biometric_repository.dart';
import '../data/repository/card/data_attribute_repository.dart';
import '../data/repository/card/impl/data_attribute_repository_impl.dart';
import '../data/repository/card/impl/wallet_card_repository_impl.dart';
import '../data/repository/card/wallet_card_repository.dart';
import '../data/repository/configuration/configuration_repository.dart';
import '../data/repository/configuration/impl/configuration_repository_impl.dart';
import '../data/repository/disclosure/core/core_disclosure_repository.dart';
import '../data/repository/disclosure/disclosure_repository.dart';
import '../data/repository/event/core/core_wallet_event_repository.dart';
import '../data/repository/event/wallet_event_repository.dart';
import '../data/repository/issuance/core/core_issuance_repository.dart';
import '../data/repository/issuance/issuance_repository.dart';
import '../data/repository/language/impl/language_repository_impl.dart';
import '../data/repository/language/language_repository.dart';
import '../data/repository/pid/core/core_pid_repository.dart';
import '../data/repository/pid/pid_repository.dart';
import '../data/repository/qr/core/core_qr_repository.dart';
import '../data/repository/qr/qr_repository.dart';
import '../data/repository/sign/core/core_sign_repository.dart';
import '../data/repository/sign/sign_repository.dart';
import '../data/repository/uri/core/core_uri_repository.dart';
import '../data/repository/uri/uri_repository.dart';
import '../data/repository/version/core/core_version_string_repository.dart';
import '../data/repository/version/version_string_repository.dart';
import '../data/repository/wallet/core/core_wallet_repository.dart';
import '../data/repository/wallet/wallet_repository.dart';

/// This widget is responsible for initializing and providing all `repositories`.
/// Most likely to be used once at the top (app) level.
class WalletRepositoryProvider extends StatelessWidget {
  final Widget child;

  const WalletRepositoryProvider({required this.child, super.key});

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<WalletRepository>(
          create: (context) => CoreWalletRepository(context.read(), context.read()),
        ),
        RepositoryProvider<WalletCardRepository>(create: (context) => WalletCardRepositoryImpl(context.read())),
        RepositoryProvider<DataAttributeRepository>(
          create: (context) => DataAttributeRepositoryImpl(context.read()),
        ),
        RepositoryProvider<DisclosureRepository>(
          create: (context) => CoreDisclosureRepository(
            context.read(),
            context.read(),
            context.read(),
            context.read(),
            context.read(),
            context.read(),
            context.read(),
            context.read(),
          ),
        ),
        RepositoryProvider<IssuanceRepository>(
          create: (context) => CoreIssuanceRepository(
            issuanceApi,
            context.read(),
            context.read(),
            context.read(),
            context.read(),
            context.read(),
          ),
        ),
        RepositoryProvider<ConfigurationRepository>(
          create: (context) => ConfigurationRepositoryImpl(context.read()),
        ),
        RepositoryProvider<QrRepository>(
          create: (context) => CoreQrRepository(context.read()),
        ),
        RepositoryProvider<LanguageRepository>(
          create: (context) => LanguageRepositoryImpl(context.read(), AppLocalizations.supportedLocales),
        ),
        RepositoryProvider<PidRepository>(
          create: (context) => CorePidRepository(context.read(), context.read()),
        ),
        RepositoryProvider<UriRepository>(
          create: (context) => CoreUriRepository(context.read()),
        ),
        RepositoryProvider<WalletEventRepository>(
          create: (context) => CoreWalletEventRepository(context.read(), context.read()),
        ),
        RepositoryProvider<BiometricRepository>(
          create: (context) => CoreBiometricRepository(context.read()),
        ),
        RepositoryProvider<SignRepository>(
          create: (context) => CoreSignRepository(
            signingApi,
            context.read(),
            context.read(),
            context.read(),
            context.read(),
          ),
        ),
        RepositoryProvider<VersionStringRepository>(
          create: (context) => CoreVersionStringRepository(context.read()),
        ),
      ],
      child: child,
    );
  }
}
