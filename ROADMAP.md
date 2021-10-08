This repository is in active development as we strive to implement features outlined in the [GraphQL Spec](https://spec.graphql.org/).

While this is a non-exhaustive list, current large features we are planning on building are as follows:

1. **Semantic Analysis and Validation**
    We will be building a separate structure on top of the the AST created by
    the parser to allow for more effient schema querying. This will include more
    easily accessible information about queries, mutations, types and their
    subtypes, directives etc.

    Validation will correspond to [#Validation part](https://spec.graphql.org/draft/#sec-Validation)
    of the GraphQL spec. This feature allows us to have spec-correct queries and
    schemas.

3. **Execution.**
    Corresponds to [#Execution part](https://spec.graphql.org/draft/#sec-Execution)
    of the GraphQL spec.  Execution has to be implemented after we implement
    validation and HIR. It will run on Operations, SelectionSets, Fields, and
    Requests.

4. **Response shaping.**
    Corresponds to [#Response part](https://spec.graphql.org/draft/#sec-Response)
    of the GraphQL spec. This bit of work will be responsible for providing an
    API to shape a Response and its errors. It may be worked in parallel with
    Execution and can only be done after HIR and validation are completed.
