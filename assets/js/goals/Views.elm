module Goals.Views exposing (editView, renderGoal, renderGoals)

import Goals.Messages exposing (..)
import Goals.Models exposing (Goal)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Model exposing (Msg(..))


editView : Goal -> Html Goals.Messages.Msg
editView model =
    div []
        [ input [ type_ "text", value model.title, onInput Goals.Messages.TitleUpdated ] []
        , input [ type_ "text", value model.startDate, onInput Goals.Messages.StartDateUpdated ] []
        , input [ type_ "text", value model.endDate, onInput Goals.Messages.EndDateUpdated ] []
        , input [ type_ "text", value (String.fromFloat model.target), onInput Goals.Messages.TargetUpdated ] []
        , button [ onClick Goals.Messages.SaveGoal ] [ text "Save" ]
        , button [ onClick Goals.Messages.DeleteGoal ] [ text "Delete" ]
        ]


renderGoals : List Goal -> Html Msg
renderGoals goals =
    div []
        [ button [ class "btn btn-primary", onClick CreateGoal ] [ text "New Goal" ]
        , div
            []
            [ div []
                [ th [] [ text "Account Name" ]
                , th [] [ text "Balance" ]
                , th [] [ text "Debt?" ]
                ]
            ]
        , div [] (List.map renderGoal goals)
        ]


renderGoal : Goal -> Html Msg
renderGoal goal =
    div []
        [ div [ onClick (OpenGoalEditor goal) ] [ text goal.title ]
        , div [] [ text (String.fromFloat goal.target) ]
        , div [] [ text goal.startDate ]
        , div [] [ text goal.endDate ]
        ]
