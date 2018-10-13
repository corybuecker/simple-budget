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
        ]


renderGoals : List Goal -> Html Msg
renderGoals goals =
    div []
        [ button [ onClick CreateGoal ] [ text "New Goal" ]
        , table
            []
            [ thead []
                [ tr []
                    [ th [] [ text "Account Name" ]
                    , th [] [ text "Balance" ]
                    , th [] [ text "Debt?" ]
                    ]
                ]
            , tbody [] (List.map renderGoal goals)
            ]
        ]


renderGoal : Goal -> Html Msg
renderGoal goal =
    tr []
        [ td [ onClick (OpenGoalEditor goal) ] [ text goal.title ]
        , td [] [ text (String.fromFloat goal.target) ]
        , td [] [ text goal.startDate ]
        , td [] [ text goal.endDate ]
        ]
