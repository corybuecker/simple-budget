module Savings.Messages exposing (Msg(..))

import Http
import Savings.Models exposing (Saving)


type Msg
    = TitleUpdated String
    | AmountUpdated String
    | SaveSaving
    | DeleteSaving
    | CreateSaving
    | OpenSavingEditor Saving
    | SavingsFetched (Result Http.Error (List Saving))
