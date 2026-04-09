import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/card/delete/bloc/delete_card_bloc.dart';

void main() {
  group('DeleteCardBloc', () {
    blocTest<DeleteCardBloc, DeleteCardState>(
      'emits [] when nothing is added',
      build: DeleteCardBloc.new,
      expect: () => [],
      verify: (bloc) => expect(bloc.state, const DeleteCardInitial()),
    );

    blocTest<DeleteCardBloc, DeleteCardState>(
      'emits [DeleteCardProvidePin] when DeleteCardLoadTriggered is added',
      build: DeleteCardBloc.new,
      act: (bloc) => bloc.add(const DeleteCardLoadTriggered(attestationId: 'card-123', cardTitle: 'Test Card')),
      expect: () => [const DeleteCardProvidePin(attestationId: 'card-123', cardTitle: 'Test Card')],
    );

    blocTest<DeleteCardBloc, DeleteCardState>(
      'emits [DeleteCardSuccess] when DeleteCardPinConfirmed is added from [DeleteCardProvidePin]',
      build: DeleteCardBloc.new,
      seed: () => const DeleteCardProvidePin(attestationId: 'card-123', cardTitle: 'Test Card'),
      act: (bloc) => bloc.add(const DeleteCardPinConfirmed()),
      expect: () => [const DeleteCardSuccess(cardTitle: 'Test Card')],
    );

    blocTest<DeleteCardBloc, DeleteCardState>(
      'does not emit when DeleteCardPinConfirmed is added from [DeleteCardInitial]',
      build: DeleteCardBloc.new,
      act: (bloc) => bloc.add(const DeleteCardPinConfirmed()),
      expect: () => [],
    );
  });
}
