module Goals.Views exposing (editView, emptyGoal)

import Goals.Messages
import Goals.Models exposing (Goal)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)


emptyGoal =
    Goal 0 "" "" "" 0


editView : Goal -> Html Goals.Messages.Msg
editView model =
    div []
        [ input [ type_ "text", value model.title, onInput Goals.Messages.NameUpdated ] []
        , input [ type_ "text", value model.startDate ] []
        , input [ type_ "text", value model.endDate ] []
        , input [ type_ "text", value (String.fromFloat model.target) ] []
        ]
