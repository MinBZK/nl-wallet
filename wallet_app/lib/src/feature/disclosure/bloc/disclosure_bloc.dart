import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../domain/usecase/card/log_card_interaction_usecase.dart';
import '../../report_issue/report_issue_screen.dart';
import '../model/disclosure_flow.dart';

part 'disclosure_event.dart';
part 'disclosure_state.dart';

class DisclosureBloc extends Bloc<DisclosureEvent, DisclosureState> {
  DisclosureBloc({DisclosureState initialState = const DisclosureInitial()}) : super(initialState);
}
