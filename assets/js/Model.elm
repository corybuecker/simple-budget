module Model exposing (Model, Msg(..))

import Accounts.Messages
import Accounts.Models exposing (Account)
import Adjustments.Messages
import Adjustments.Models exposing (Adjustment)
import Browser
import Browser.Navigation
import Goals.Messages
import Goals.Models exposing (Goal)
import Http
import Savings.Messages
import Savings.Models exposing (Saving)
import Url


type Msg
    = AccountsFetched (Result Http.Error (List Account))
    | GoalsFetched (Result Http.Error (List Goal))
    | SavingsFetched (Result Http.Error (List Saving))
    | AdjustmentsFetched (Result Http.Error (List Adjustment))
    | OpenAccountEditor Account
    | OpenGoalEditor Goal
    | OpenSavingEditor Saving
    | OpenAdjustmentEditor Adjustment
    | UpdateAccount Accounts.Messages.Msg
    | UpdateGoal Goals.Messages.Msg
    | UpdateSaving Savings.Messages.Msg
    | UpdateAdjustment Adjustments.Messages.Msg
    | UrlRequest Browser.UrlRequest
    | UrlChanged Url.Url
    | CreateGoal
    | CreateSaving
    | CreateAccount
    | CreateAdjustment Account


type alias Model =
    { accounts : List Account
    , goals : List Goal
    , savings : List Saving
    , error : Maybe Http.Error
    , modalOpen : String
    , activeAccount : Maybe Account
    , activeGoal : Maybe Goal
    , activeSaving : Maybe Saving
    , activeAdjustment : Maybe Adjustment
    , key : Browser.Navigation.Key
    , page : String
    }
