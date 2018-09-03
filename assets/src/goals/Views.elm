module Goals.Views exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Goals.Models exposing (Goal)
import Goals.Messages


emptyGoal =
    Goal 0 "" False 0


editView : Goal -> Html Goals.Messages.Msg
editView model =
    div []
        [ input [ type_ "text", value model.name, onInput Goals.Messages.NameUpdated ] []
        , input [ type_ "checkbox", checked model.debt ] []
        , input [ type_ "text", value (String.fromFloat model.balance) ] []
        ]
