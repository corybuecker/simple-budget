module Model exposing (Model, Msg(..))

import Accounts.Messages
import Accounts.Models exposing (Account)
import Browser
import Browser.Navigation
import Goals.Messages
import Goals.Models exposing (Goal)
import Http
import Url


type Msg
    = AccountsFetched (Result Http.Error (List Account))
    | GoalsFetched (Result Http.Error (List Goal))
    | OpenAccountEditor Account
    | OpenGoalEditor Goal
    | UpdateAccount Accounts.Messages.Msg
    | UpdateGoal Goals.Messages.Msg
    | UrlRequest Browser.UrlRequest
    | UrlChanged Url.Url
    | CreateGoal


type alias Model =
    { accounts : List Account
    , goals : List Goal
    , error : Maybe Http.Error
    , modalOpen : String
    , activeAccount : Account
    , activeGoal : Goal
    , key : Browser.Navigation.Key
    , page : String
    }
