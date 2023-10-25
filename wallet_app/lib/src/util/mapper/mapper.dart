abstract class Mapper<I, O> {
  O map(I input);

  List<O> mapList(List<I> input) => input.map((e) => map(e)).toList();
}
