module Savings.Messages exposing (Msg(..))

import Http
import Savings.Models exposing (Saving)


type Msg
    = TitleUpdated String
    | AmountUpdated String
    | SaveSaving
    | DeleteSaving
