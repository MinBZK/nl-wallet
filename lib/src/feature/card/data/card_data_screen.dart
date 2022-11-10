import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/data_attribute.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/data_attribute_image.dart';
import '../../common/widget/data_attribute_row.dart';
import 'bloc/card_data_bloc.dart';

class CardDataScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a (mock) id when opening the CardSummaryScreen');
    }
  }

  const CardDataScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).cardDataAttributesScreenTitle),
      ),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    return BlocBuilder<CardDataBloc, CardDataState>(
      builder: (context, state) {
        if (state is CardDataInitial) return _buildLoading();
        if (state is CardDataLoadInProgress) return _buildLoading();
        if (state is CardDataLoadSuccess) return _buildDataAttributes(state.attributes);
        throw UnsupportedError('Unknown state: $state');
      },
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildDataAttributes(List<DataAttribute> attributes) {
    return ListView.separated(
      padding: const EdgeInsets.all(16.0),
      itemBuilder: (context, index) {
        if (index < attributes.length) {
          return _buildDataAttribute(attributes[index]);
        } else {
          return _buildFooterButton(context);
        }
      },
      separatorBuilder: (context, index) => const SizedBox(height: 16.0),
      itemCount: attributes.length + 1,
    );
  }

  Widget _buildDataAttribute(DataAttribute attribute) {
    assert(attribute.value != null, 'CardDataScreen does not support incomplete datasets');
    if (attribute.type == 'Image') {
      return Align(
        alignment: Alignment.centerLeft,
        child: DataAttributeImage(image: AssetImage(attribute.value!)),
      );
    } else {
      return DataAttributeRow(attribute: attribute);
    }
  }

  Widget _buildFooterButton(BuildContext context) {
    return TextButton(
      onPressed: () => _onCloseButtonPressed(context),
      child: Text(AppLocalizations.of(context).cardDataAttributesCloseButton),
    );
  }

  void _onCloseButtonPressed(BuildContext context) {
    Navigator.pop(context);
  }
}
