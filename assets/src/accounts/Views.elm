module Accounts.Views exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Accounts.Models exposing (Account)
import Accounts.Messages


emptyAccount =
    Account 0 "" False 0 []


editView : Account -> Html Accounts.Messages.Msg
editView model =
    div []
        [ input [ type_ "text", value model.name, onInput Accounts.Messages.NameUpdated ] []
        , input [ type_ "checkbox", checked model.debt ] []
        , input [ type_ "text", value (String.fromFloat model.balance) ] []
        ]
