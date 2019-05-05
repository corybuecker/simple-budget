module Accounts.Messages exposing (Msg(..))

import Accounts.Models exposing (..)
import Http exposing (Error)


type Msg
    = NameUpdated String
    | ToggleDebt
    | BalanceUpdated String
    | SaveAccount
    | DeleteAccount Account
    | ToggleAdjustmentsFor Account
    | AccountsFetched (Result Http.Error (List Account))
    | AdjustmentsFetched (Result Http.Error (List Adjustment))
    | OpenAccountEditor Account
    | OpenAdjustmentEditor Adjustment
    | CreateAccount
    | CreateAdjustment Account
    | TitleUpdated String
    | TotalUpdated String
    | SaveAdjustment
    | DeleteAdjustment Adjustment
    | DeleteCompleted (Result Http.Error ())
    | AccountSaved (Result Http.Error Account)
    | AdjustmentSaved (Result Http.Error Adjustment)
