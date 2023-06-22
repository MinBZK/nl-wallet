import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../bridge_generated.dart';
import '../../../wallet_core/typed_wallet_core.dart';
import 'centered_loading_indicator.dart';

class FlutterConfigurationProvider extends StatelessWidget {
  /// This builder will be called with the latest [FlutterConfiguration]
  final Widget Function(FlutterConfiguration) builder;

  /// The default [FlutterConfiguration] to be provided to the [builder].
  /// When this is null a [CenteredLoadingIndicator] will be rendered until
  /// a [FlutterConfiguration] is available.
  final FlutterConfiguration? defaultConfig;

  /// The stream to be observed to fetch the [FlutterConfiguration], when
  /// set to null the provider is fetched through the [TypedWalletCore],
  /// mainly used for testing.
  final Stream<FlutterConfiguration>? configProvider;

  const FlutterConfigurationProvider({
    required this.builder,
    this.configProvider,
    this.defaultConfig,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<FlutterConfiguration>(
      stream: configProvider ?? context.read<TypedWalletCore>().observeConfig(),
      builder: (context, snapshot) {
        if (!snapshot.hasData && defaultConfig == null) return const CenteredLoadingIndicator();
        Fimber.i('Providing config: ${snapshot.data ?? defaultConfig}');
        return builder(snapshot.data ?? defaultConfig!);
      },
    );
  }
}
