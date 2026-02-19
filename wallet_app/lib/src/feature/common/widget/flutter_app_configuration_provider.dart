import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repository/configuration/configuration_repository.dart';
import '../../../domain/model/configuration/flutter_app_configuration.dart';
import '../../../wallet_core/typed/typed_wallet_core.dart';
import 'centered_loading_indicator.dart';

class FlutterAppConfigurationProvider extends StatelessWidget {
  /// This builder will be called with the latest [FlutterAppConfiguration]
  final Widget Function(FlutterAppConfiguration) builder;

  /// The default [FlutterAppConfiguration] to be provided to the [builder].
  /// When this is null a [CenteredLoadingIndicator] will be rendered until
  /// a [FlutterAppConfiguration] is available.
  final FlutterAppConfiguration? defaultConfig;

  /// The stream to be observed to fetch the [FlutterAppConfiguration], when
  /// set to null the provider is fetched through the [TypedWalletCore],
  /// mainly used for testing.
  final Stream<FlutterAppConfiguration>? configProvider;

  const FlutterAppConfigurationProvider({
    required this.builder,
    this.configProvider,
    this.defaultConfig,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<FlutterAppConfiguration>(
      stream: configProvider ?? context.read<ConfigurationRepository>().observeAppConfiguration,
      builder: (context, snapshot) {
        if (!snapshot.hasData && defaultConfig == null) return const CenteredLoadingIndicator();
        Fimber.i('Providing config: ${snapshot.data ?? defaultConfig}');
        return builder(snapshot.data ?? defaultConfig!);
      },
    );
  }
}
