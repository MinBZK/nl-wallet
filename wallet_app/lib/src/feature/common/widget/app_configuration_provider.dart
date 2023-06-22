import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repository/configuration/configuration_repository.dart';
import '../../../domain/model/configuration/app_configuration.dart';
import '../../../wallet_core/typed_wallet_core.dart';
import 'centered_loading_indicator.dart';

class AppConfigurationProvider extends StatelessWidget {
  /// This builder will be called with the latest [AppConfiguration]
  final Widget Function(AppConfiguration) builder;

  /// The default [AppConfiguration] to be provided to the [builder].
  /// When this is null a [CenteredLoadingIndicator] will be rendered until
  /// a [AppConfiguration] is available.
  final AppConfiguration? defaultConfig;

  /// The stream to be observed to fetch the [AppConfiguration], when
  /// set to null the provider is fetched through the [TypedWalletCore],
  /// mainly used for testing.
  final Stream<AppConfiguration>? configProvider;

  const AppConfigurationProvider({
    required this.builder,
    this.configProvider,
    this.defaultConfig,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<AppConfiguration>(
      stream: configProvider ?? context.read<ConfigurationRepository>().appConfiguration,
      builder: (context, snapshot) {
        if (!snapshot.hasData && defaultConfig == null) return const CenteredLoadingIndicator();
        Fimber.i('Providing config: ${snapshot.data ?? defaultConfig}');
        return builder(snapshot.data ?? defaultConfig!);
      },
    );
  }
}
