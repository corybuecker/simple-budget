port module Login.Google exposing (login, useIdToken)

import Json.Encode exposing (Value)


port login : () -> Cmd msg


port useIdToken : (Value -> msg) -> Sub msg
