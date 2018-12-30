module Goals.Views exposing (editView, renderGoal, renderGoals)

import Goals.Messages exposing (..)
import Goals.Models exposing (Goal)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import List exposing (map)
import List.Extra exposing (greedyGroupsOf)


editView : Goal -> Html Msg
editView model =
    div []
        [ input [ type_ "text", value model.title, onInput TitleUpdated ] []
        , input [ type_ "text", value model.startDate, onInput StartDateUpdated ] []
        , input [ type_ "text", value model.endDate, onInput EndDateUpdated ] []
        , input [ type_ "text", value (String.fromFloat model.target), onInput TargetUpdated ] []
        , button [ onClick SaveGoal ] [ text "Save" ]
        , button [ onClick DeleteGoal ] [ text "Delete" ]
        ]


renderGoals : List Goal -> Html Msg
renderGoals goals =
    div []
        [ button [ class "btn btn-primary", onClick CreateGoal ] [ text "New Goal" ]
        , div [] (map renderGoalGroup (greedyGroupsOf 2 goals))
        ]


renderGoalGroup : List Goal -> Html Msg
renderGoalGroup group =
    div [ class "row" ] (map renderGoal group)


renderGoal : Goal -> Html Msg
renderGoal goal =
    div [ class "col-sm-6" ]
        [ div [ class "card" ]
            [ div [ class "card-body" ]
                [ h5 [ class "card-title", onClick (OpenGoalEditor goal) ] [ text goal.title ]
                , div [] [ text (String.fromFloat goal.target) ]
                , div [] [ text goal.startDate ]
                , div [] [ text goal.endDate ]
                ]
            ]
        ]
